const getModeImgUrl = (mode) => {
  const m = {
    open: "bankara",
    challenge: "bankara",
    x: "x",
    event: "event"
  }
  return `/img/mode/${m[mode]}.svg`
}

const getRuleImgUrl = (rule) => {
  return `/img/rule/${rule}.svg`
}

const getPVPStageImgUrl = (stage) => {
  return `/img/stage/vs/${btoa(`VsStage-${stage}`)}.png`
}

const getCoopStageImgUrl = (stage) => {
  return `/img/stage/coop/${btoa(`CoopStage-${stage}`)}.png`
}

const getBrowserImgUrl = (key) => {
  const m = {
    'chrome': 'chrome.svg',
    'firefox': 'firefox.svg',
    'safari': 'safari.png',
    'edge': 'edge.png',
  }
  return `/img/browser/${m[key.toLowerCase()] || 'chromium.svg'}`
}

export {
  getModeImgUrl,
  getRuleImgUrl,
  getPVPStageImgUrl,
  getCoopStageImgUrl,
  getBrowserImgUrl,
}
