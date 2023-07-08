import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  // hashbang: false,
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: () => import("./pages/Home.vue"),
    },
    {
      path: '/login/google',
      component: () => import("./auth/Google.vue"),
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
      component: () => import("./pages/ListQuery.vue"),
      meta: {
        auth: true,
      }
    },
    {
      path: '/query/edit',
      component: () => import("./pages/EditQuery.vue"),
      props: route => ({ qtype: route.query.qtype, qid: parseInt(route.query.qid) }),
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
