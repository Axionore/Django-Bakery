import { createBrowserRouter } from "react-router";

import { RootLayout } from "~/routes/_layout";
import { HomePage } from "~/routes/index";
import { AboutPage } from "~/routes/about";
import { NotFoundPage } from "~/routes/_not-found";
import { AccountLayout } from "~/routes/account/_layout";
import { LoginPage } from "~/routes/account/login";
import { SignupPage } from "~/routes/account/signup";
import { ProfilePage } from "~/routes/account/profile";
import { VerifyEmailPage } from "~/routes/account/verify-email";
import { MfaChallengePage } from "~/routes/account/mfa-challenge";
import { MfaActivatePage } from "~/routes/account/mfa-activate";
import { RecoveryCodesPage } from "~/routes/account/recovery-codes";

export const router = createBrowserRouter([
  {
    path: "/",
    Component: RootLayout,
    children: [
      { index: true, Component: HomePage },
      { path: "about", Component: AboutPage },
      {
        path: "account",
        Component: AccountLayout,
        children: [
          { path: "login", Component: LoginPage },
          { path: "signup", Component: SignupPage },
          { path: "profile", Component: ProfilePage },
          { path: "verify-email", Component: VerifyEmailPage },
          { path: "mfa-challenge", Component: MfaChallengePage },
          { path: "mfa-activate", Component: MfaActivatePage },
          { path: "recovery-codes", Component: RecoveryCodesPage },
        ],
      },
      { path: "*", Component: NotFoundPage },
    ],
  },
]);
