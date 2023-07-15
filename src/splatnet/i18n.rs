use super::{PvpMode, PvpRule};

pub trait I18N: Send + Sync {
  fn get_pvp_mode_name(&self, mode: PvpMode) -> String;
  fn get_pvp_rule_name(&self, rule: PvpRule) -> String;
  fn get_pvp_stage_name(&self, id: u32) -> String;
}

pub struct EnUs();

impl I18N for EnUs {
  fn get_pvp_mode_name(&self, mode: PvpMode) -> String {
    mode.to_string()
  }

  fn get_pvp_rule_name(&self, rule: PvpRule) -> String {
    rule.to_string()
  }

  fn get_pvp_stage_name(&self, id: u32) -> String {
    let x = &[
      "?",
      "Scorch Gorge",
      "Eeltail Alley",
      "Hagglefish Market",
      "Undertow Spillway",
      "Um'ami Ruins",
      "Mincemeat Metalworks",
      "Brinewater Springs",
      "Barnacle & Dime",
      "Flounder Heights",
      "Hammerhead Bridge",
      "Museum d'Alfonsino",
      "Mahi-Mahi Resort",
      "Inkblot Art Academy",
      "Sturgeon Shipyard",
      "MakoMart",
      "Wahoo World",
      "Humpback Pump Track",
      "Manta Maria",
    ];
    if (id as usize) < x.len() {
      x[id as usize].into()
    } else {
      "?".into()
    }
  }
}
