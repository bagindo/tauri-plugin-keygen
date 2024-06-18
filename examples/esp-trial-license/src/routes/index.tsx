import { Navigate, createFileRoute } from "@tanstack/react-router";
import { formatISO } from "date-fns";

export const Route = createFileRoute("/")({
  component: () => (
    <Navigate
      to="/esp"
      search={{ date: formatISO(new Date(), { representation: "date" }) }}
    />
  ),
});
