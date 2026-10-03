import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";
import { Button } from "@/components/ui/button";
import {
  ErrorActions,
  ErrorDescription,
  ErrorHeader,
  ErrorView,
} from "@/features/errors/error-base";

export default function NotFoundErrorPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const goBack = useCallback(() => navigate(-1), [navigate]);
  return (
    <ErrorView>
      <ErrorHeader>{t("errors.notFound.title")}</ErrorHeader>
      <ErrorDescription>{t("errors.notFound.description")}</ErrorDescription>
      <ErrorActions>
        <Button onClick={goBack} size="lg">
          {t("errors.notFound.action")}
        </Button>
      </ErrorActions>
    </ErrorView>
  );
}
