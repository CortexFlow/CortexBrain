import React from "react";
import { createBrowserRouter } from "react-router-dom";
import App from "../App";

const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <App />,
/*       errorElement: <ErrorPage />, */
    },
  ],
);

export default router;
