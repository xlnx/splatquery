const getModeImgUrl = (mode) => {
  const m = {
    open: "bankara",
    challenge: "bankara",
  }
  return `/img/mode/${m[mode] || mode}.svg`
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

const invalidateCache = async (cacheName, url) => {
  try {
    const registration = await navigator.serviceWorker.getRegistration();
    registration.active.postMessage({ type: 'invalidateCache', params: { cacheName, url } });
  } catch (err) { }
}

export {
  getModeImgUrl,
  getRuleImgUrl,
  getPVPStageImgUrl,
  getCoopStageImgUrl,
  getBrowserImgUrl,
  invalidateCache,
}
