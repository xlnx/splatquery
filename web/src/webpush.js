import { registerSW } from 'virtual:pwa-register'

const tryGetSubInfo = async () => {
  try {
    const registration = await navigator.serviceWorker.getRegistration();
    return await registration.pushManager.getSubscription();
  } catch (err) {
    return null;
  }
}

const getSubInfo = async () => {
  const { Notification } = window;
  if (!Notification) {
    throw 'Notification is not supported by your browser.';
  }
  try {
    await Notification.requestPermission();
  } catch (err) {
    throw 'Notification permission denied.'
  }
  const { serviceWorker } = navigator;
  if (!serviceWorker) {
    throw 'You have to quit private mode to use webpush, or maybe your browser doesn\'t support service workers.'
  }
  let registration = null;
  try {
    await registerSW()();
    registration = await serviceWorker.getRegistration();
    if (!registration) { throw null; }
  } catch (err) {
    throw 'Register service worker failed, please contact the developer for help.'
  }
  const { pushManager } = registration;
  if (!pushManager) {
    throw 'WebPush is not supported by your browser.'
  }
  let subInfo = null;
  try {
    subInfo = await pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: "BDKNzkxVCQM1T131qz1Ctoz3f8t2sNge-uD7D216Wi1rrVaOYfl1r_ZYNKD2LgYAVWjXVZdUHvU0BNnVhdGJSA0",
    });
  } catch (err) {
    throw 'WebPush permission denied.'
  }
  return subInfo.toJSON();
}

export {
  tryGetSubInfo,
  getSubInfo,
}
