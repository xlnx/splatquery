import { createRouter, createWebHistory } from 'vue-router';
import { Base64 } from 'js-base64';

const router = createRouter({
  // hashbang: false,
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: (
        window.matchMedia('(display-mode: standalone)').matches ?
          () => import("./pages/Pwa.vue") : () => import("./pages/Home.vue")
      )
    },
    {
      path: '/login/google',
      component: () => import("./auth/Google.vue"),
    },
    {
      path: '/settings',
      component: () => import("./pages/Settings.vue"),
      meta: {
        auth: true,
      }
    },
    {
      path: '/query/new',
      component: () => import("./pages/NewQuery.vue"),
      meta: {
        auth: true,
      }
    },
    {
      path: '/query/list',
      component: () => import("./pages/ListQueries.vue"),
      meta: {
        auth: true,
      }
    },
    {
      path: '/query/edit',
      component: () => import("./pages/EditQuery.vue"),
      props: route => {
        const { qid, config } = JSON.parse(Base64.decode(route.query.query));
        return { qid, query: config };
      },
      meta: {
        auth: true,
      }
    },
    {
      path: '/action/list',
      component: () => import("./pages/ListActions.vue"),
      meta: {
        auth: true,
      }
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/',
    }
  ]
});

export default (app) => {
  app.router = router;

  app.use(router);
}
