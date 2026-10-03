import { createBrowserRouter, RouterProvider } from "react-router";
import { HomePage } from "@/app/routes/home";
import NotFoundErrorPage from "@/app/routes/not-found";

const router = createBrowserRouter([
  { Component: HomePage, path: "/" },
  { Component: NotFoundErrorPage, path: "*" },
]);

export default function AppRouter() {
  return <RouterProvider router={router} />;
}
