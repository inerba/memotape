import { createBrowserRouter, RouterProvider } from "react-router";
import { HomePage } from "@/app/routes/home";
import NotFoundErrorPage from "@/app/routes/not-found";
import { SettingsPage } from "@/app/routes/settings";

const router = createBrowserRouter([
  // Impostazioni si apre sopra la finestra principale, che resta montata: il testo e una
  // Trascrizione in corso non si perdono.
  {
    Component: HomePage,
    children: [{ Component: SettingsPage, path: "settings" }],
    path: "/",
  },
  { Component: NotFoundErrorPage, path: "*" },
]);

export default function AppRouter() {
  return <RouterProvider router={router} />;
}
