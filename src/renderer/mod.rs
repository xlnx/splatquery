use std::{
  fs::File,
  io::BufWriter,
  path::PathBuf,
  sync::{Arc, RwLock},
};

use chrono::{DateTime, Datelike, Duration, Timelike};
use image::{codecs::jpeg::JpegEncoder, ColorType, ImageEncoder};
use itertools::Itertools;
use minijinja::{context, path_loader, Environment};
use resvg::{
  tiny_skia::Pixmap,
  usvg::{fontdb, Options, Transform, Tree, TreeParsing, TreeTextToPath},
};
use serde::{Deserialize, Serialize};
use ttl_cache::TtlCache;
use walkdir::WalkDir;

use crate::{
  database::{Language, TimeZone},
  splatnet::PVPSpiderItem,
  BoxError,
};

#[derive(Deserialize)]
pub struct RendererConfig {
  pub out_dir: String,
  pub assets_dir: String,
  #[serde(default)]
  pub font_family: String,
  #[serde(default = "default_cache_size")]
  pub cache_size: usize,
}

fn default_cache_size() -> usize {
  1024
}

pub struct RenderOptions<'a> {
  pub platform: &'a str,
  pub language: Language,
  pub time_zone: TimeZone,
}

pub struct Renderer {
  out_dir: PathBuf,
  svg_opts: Options,
  svg_tmpls: Environment<'static>,
  fontdb: fontdb::Database,
  lookup: RwLock<TtlCache<String, String>>,
}

impl Renderer {
  pub fn new(opts: RendererConfig) -> Result<Arc<Self>, BoxError> {
    let mut svg_opts = Options::default();
    svg_opts.resources_dir = Some(opts.assets_dir.clone().into());
    svg_opts.font_family = opts.font_family.into();
    let mut svg_tmpls = Environment::new();
    svg_tmpls.set_loader(path_loader(
      [opts.assets_dir.as_str(), "svg"]
        .iter()
        .collect::<PathBuf>(),
    ));
    let mut fontdb = fontdb::Database::new();
    let mut iter = WalkDir::new(
      [opts.assets_dir.as_str(), "font"]
        .iter()
        .collect::<PathBuf>(),
    )
    .into_iter();
    while let Some(Ok(entry)) = iter.next() {
      if entry.path().is_file() {
        if let Err(err) = fontdb.load_font_file(entry.path()) {
          log::warn!("load font file [{:?}] failed: [{:?}]", entry.path(), err);
        }
      }
    }
    Ok(Arc::new(Renderer {
      out_dir: opts.out_dir.into(),
      svg_opts,
      svg_tmpls,
      fontdb,
      lookup: RwLock::new(TtlCache::new(opts.cache_size)),
    }))
  }

  pub fn out_dir(&self) -> String {
    self.out_dir.clone().into_os_string().into_string().unwrap()
  }

  pub fn render_pvp(&self, item: &PVPSpiderItem, opts: &RenderOptions) -> Result<String, BoxError> {
    self.render(
      &format!("pvp.{}", opts.platform),
      || {
        let stages: Vec<_> = item
          .stages
          .iter()
          .map(|s| base64::encode(format!("VsStage-{}", s)))
          .collect();
        context!(
          mode => item.mode.to_string(),
          rule => item.rule.to_string(),
          stages => stages,
          start_time => &fmt_time_range_2h(item.start_time, opts.time_zone),
        )
      },
      &[
        &opts.time_zone.to_string(),
        &item.mode.to_string(),
        &item.start_time.day().to_string(),
        &item.start_time.hour().to_string(),
      ],
    )
  }

  fn render<S, Ctx>(&self, tmpl: &str, ctx: S, keys: &[&str]) -> Result<String, BoxError>
  where
    S: FnOnce() -> Ctx,
    Ctx: Serialize,
  {
    let key = [tmpl].iter().chain(keys).join(".");
    if let Some(path) = self.lookup.read().unwrap().get(&key) {
      return Ok(path.clone());
    }
    let mut cache = self.lookup.write().unwrap();
    if let Some(path) = cache.get(&key) {
      return Ok(path.clone());
    }
    let pixmap = self.do_render(tmpl, ctx())?;
    let path = base64::encode_config(&key, base64::URL_SAFE_NO_PAD) + ".jpg";
    let file = File::create(&self.out_dir.join(path.clone())).unwrap();
    let buff = BufWriter::new(file);
    let encoder = JpegEncoder::new(buff);
    encoder.write_image(
      pixmap.data(),
      pixmap.width(),
      pixmap.height(),
      ColorType::Rgba8,
    )?;
    let ttl = Duration::days(2).to_std()?;
    cache.insert(key, path.clone(), ttl);
    Ok(path)
  }

  fn do_render<S: Serialize>(&self, tmpl: &str, ctx: S) -> Result<Pixmap, BoxError> {
    let tmpl = self
      .svg_tmpls
      .get_template([tmpl, ".jinja"].join("").as_str())?;
    let svg = tmpl.render(&ctx)?;
    let mut tree = Tree::from_str(&svg, &self.svg_opts)?;
    tree.convert_text(&self.fontdb);
    let tree = resvg::Tree::from_usvg(&tree);
    let mut pixmap = Pixmap::new(tree.size.width() as u32, tree.size.height() as u32).unwrap();
    tree.render(Transform::default(), &mut pixmap.as_mut());
    Ok(pixmap)
  }

  // pub async fn get(&self, id: &str) -> Option<Arc<Vec<u8>>> {}
}

fn fmt_time_range_2h<T>(st: DateTime<T>, tz: TimeZone) -> String
where
  T: chrono::TimeZone,
{
  let st = tz.convert(st);
  let mo = st.month();
  let md = st.day();
  match tz {
    TimeZone::Pt | TimeZone::Cest => {
      // let et = st.add(chrono::Duration::hours(2));
      let (st0, st1) = st.hour12();
      let (et0, et1) = (st + chrono::Duration::hours(2)).hour12();
      let [st0, et0] = [st0, et0].map(|t| ["AM", "PM"][t as usize]);
      format!(
        "{mo}/{md}. {st1} {st0} - {et1} {et0} {tz}",
        tz = tz.to_string().to_uppercase(),
      )
    }
    TimeZone::Jst | TimeZone::Cst => {
      let st1 = st.hour();
      let et1 = st1 + 2;
      if tz == TimeZone::Jst {
        format!(
          "{mo}/{md}({wd}) {st1}:00 - {et1}:00",
          wd = ["月", "水", "火", "木", "金", "土", "日"][st.weekday() as usize]
        )
      } else {
        format!("{mo}/{md} {st1}:00 ~ {et1}:00")
      }
    }
  }
}
