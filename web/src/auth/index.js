import { createAuth } from "@websanova/vue-auth";
import axios from 'axios';
import authBearer from '@websanova/vue-auth/dist/drivers/auth/bearer.esm.js';
import httpAxios from '@websanova/vue-auth/dist/drivers/http/axios.1.x.esm.js';
import driverRouterVueRouter from '@websanova/vue-auth/src/drivers/router/vue-router.2.x.js';
import driverOAuth2Google from '@websanova/vue-auth/dist/drivers/oauth2/google.esm.js';

export default (app) => {
  const auth = createAuth({
    plugins: {
      http: axios,
      router: app.router,
    },
    drivers: {
      http: httpAxios,
      auth: authBearer,
      router: driverRouterVueRouter,
      oauth2: {
        google: driverOAuth2Google
      }
    },
    options: {
      authRedirect: { path: '/' },
      notFoundRedirect: { path: '/' },

      loginData: { fetchUser: false },
      fetchData: { enabled: false },
      refreshData: { enabled: false },
      oauth2Data: { fetchUser: false },
    }
  });
  app.use(auth);
}
