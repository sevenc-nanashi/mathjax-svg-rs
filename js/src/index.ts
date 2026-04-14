import "./buffer.ts"
import { mathjax } from "@mathjax/src/js/mathjax.js";
import { TeX } from "@mathjax/src/js/input/tex.js";
import { liteAdaptor } from "@mathjax/src/js/adaptors/liteAdaptor.js";
import { RegisterHTMLHandler } from "@mathjax/src/js/handlers/html.js";
import { SVG } from "@mathjax/src/js/output/svg.js";
import { LiteElement } from "@mathjax/src/js/adaptors/lite/Element.js";
import { DOMParser } from "linkedom";

import "@mathjax/src/js/input/tex/base/BaseConfiguration.js";
import "@mathjax/src/js/input/tex/ams/AmsConfiguration.js";
import "@mathjax/src/js/input/tex/newcommand/NewcommandConfiguration.js";
import "@mathjax/src/js/input/tex/noundefined/NoUndefinedConfiguration.js";

/** 0: "trace", 1: "debug", 2: "info", 3: "warn", 4: "error" */
declare function __host_log(level: 0 | 1 | 2 | 3 | 4, message: string): void;

function log(level: 0 | 1 | 2 | 3 | 4, ...args: any[]) {
  const message = args
    .map((arg) => {
      if (typeof arg === "string") {
        return arg;
      } else {
        try {
          return JSON.stringify(arg);
        } catch (e) {
          return String(arg);
        }
      }
    })
    .join(" ");
  __host_log(level, message);
}
// @ts-expect-error I want to bring my own console implementation
globalThis.console = {
  trace: (...args: any[]) => log(0, ...args),
  debug: (...args: any[]) => log(1, ...args),
  info: (...args: any[]) => log(2, ...args),
  warn: (...args: any[]) => log(3, ...args),
  error: (...args: any[]) => log(4, ...args),
};

const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

const tex = new TeX({
  packages: ["base", "ams", "newcommand", "noundefined"],
});

const svg = new SVG({ fontCache: "local" });

const mathJax = mathjax.document("", {
  InputJax: tex,
  OutputJax: svg,
});

function renderTeX(math: string, fontSize: number): string {
  if (
    typeof fontSize !== "number" ||
    !Number.isFinite(fontSize) ||
    fontSize <= 0
  ) {
    throw new Error(`Font size must be positive and finite: ${fontSize}`);
  }

  const mathItem: LiteElement = mathJax.convert(math, {
    display: true,
    em: fontSize,
  });
  const item = adaptor.innerHTML(mathItem);
  if (item.includes('data-mml-node="merror"')) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(item, "text/html");
    const error = doc.querySelector("g[data-mml-node='merror']");
    if (error) {
      const message = error.getAttribute("data-mjx-error") || "Unknown error";
      throw new Error(`MathJax error: ${message}`);
    }
  }
  return item;
}

globalThis.__entry_renderTeX = renderTeX;
