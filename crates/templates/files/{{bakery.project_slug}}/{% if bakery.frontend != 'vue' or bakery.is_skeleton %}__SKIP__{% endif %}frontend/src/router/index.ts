import { createRouter, createWebHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";

import { installAuthGuards } from "~/auth/guards";

// Lazy-loaded routes split each view into its own chunk for bundle size.
const routes: RouteRecordRaw[] = [
  {
    path: "/",
    component: () => import("~/views/HomeView.vue"),
    meta: { layout: "default" },
  },
  {
    path: "/about",
    component: () => import("~/views/AboutView.vue"),
    meta: { layout: "default" },
  },
  {
    path: "/account/login",
    component: () => import("~/views/account/LoginView.vue"),
    meta: { layout: "account", guest: true },
  },
  {
    path: "/account/signup",
    component: () => import("~/views/account/SignupView.vue"),
    meta: { layout: "account", guest: true },
  },
  {
    path: "/account/verify-email",
    component: () => import("~/views/account/VerifyEmailView.vue"),
    meta: { layout: "account" },
  },
  {
    path: "/account/mfa-challenge",
    component: () => import("~/views/account/MfaChallengeView.vue"),
    meta: { layout: "account" },
  },
  {
    path: "/account/profile",
    component: () => import("~/views/account/ProfileView.vue"),
    meta: { layout: "default", requiresAuth: true },
  },
  {
    path: "/account/mfa-activate",
    component: () => import("~/views/account/MfaActivateView.vue"),
    meta: { layout: "default", requiresAuth: true },
  },
  {
    path: "/account/recovery-codes",
    component: () => import("~/views/account/RecoveryCodesView.vue"),
    meta: { layout: "default", requiresAuth: true },
  },
  {
    path: "/:pathMatch(.*)*",
    component: () => import("~/views/NotFoundView.vue"),
    meta: { layout: "default" },
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, from, saved) {
    if (saved) return saved;
    if (to.hash) return { el: to.hash };
    return { top: 0 };
  },
});

installAuthGuards(router);
