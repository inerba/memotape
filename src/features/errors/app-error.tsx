import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  ErrorActions,
  ErrorDescription,
  ErrorHeader,
  ErrorView,
} from "@/features/errors/error-base";

function reload() {
  window.location.reload();
}

export default function AppErrorPage() {
  const { t } = useTranslation();
  return (
    <ErrorView>
      <ErrorHeader>{t("errors.app.title")}</ErrorHeader>
      <ErrorDescription>{t("errors.app.description")}</ErrorDescription>
      <ErrorActions>
        <Button onClick={reload} size="lg">
          {t("errors.app.action")}
        </Button>
      </ErrorActions>
    </ErrorView>
  );
}
