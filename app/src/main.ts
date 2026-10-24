import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";
import "./canvas.css";

const target = document.getElementById("app");
if (target === null) {
  throw new Error("missing #app element");
}

const app = mount(App, { target });

export default app;
