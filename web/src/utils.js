const getModeImgUrl = (mode) => {
  const m = {
    open: "bankara",
    challenge: "bankara",
    x: "x",
    event: "event"
  }
  return `/src/assets/img/modes/${m[mode]}.svg`
}

const getRuleImgUrl = (rule) => {
  return `/src/assets/img/rules/${rule}.svg`
}

const getPVPStageImgUrl = (stage) => {
  return `/src/assets/img/stages/vs/${btoa(`VsStage-${stage}`)}.png`
}

const getCoopStageImgUrl = (stage) => {
  return `/src/assets/img/stages/coop/${btoa(`CoopStage-${stage}`)}.png`
}

export {
  getModeImgUrl,
  getRuleImgUrl,
  getPVPStageImgUrl,
  getCoopStageImgUrl,
}
