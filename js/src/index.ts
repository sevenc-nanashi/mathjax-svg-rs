/// <reference types="vite/client" />
import "./bootstrap.ts";

import MathJax from "@mathjax/src";
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
import "@mathjax/src/components/js/output/svg/svg.js";

async function init() {
  await MathJax.init({});
}

const fonts = import.meta.glob(
  "../node_modules/@mathjax/mathjax-newcm-font/svg/dynamic/*.js",
);
if (Object.keys(fonts).length === 0) {
  throw new Error(
    "No fonts found. Make sure @mathjax/mathjax-newcm-font is installed.",
  );
}
mathjax.asyncLoad = async (file: string) => {
  if (file.startsWith("@mathjax/mathjax-newcm-font/js/svg/dynamic/")) {
    const font = fonts[`../node_modules/${file.replace("/js/svg", "/svg")}`];
    if (font) {
      console.debug(`Loading font: ${file}`);
      console.debug(globalThis.MathJax);
      console.debug(globalThis.MathJax?._);
      console.debug(globalThis.MathJax?._?.output);
      console.debug(globalThis.MathJax?._?.output?.fonts);
      try {
        return await font();
      } catch (e) {
        throw new Error(
          `Failed to load font ${file}: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    } else {
      new Error(`Unknown font: ${file}`);
    }
  } else {
    new Error(`Unknown file: ${file}`);
  }
};

/** 0: "trace", 1: "debug", 2: "info", 3: "warn", 4: "error" */
declare function __host_log(level: 0 | 1 | 2 | 3 | 4, message: string): void;

function log(level: 0 | 1 | 2 | 3 | 4, ...args: any[]) {
  const message = args
    .map((arg) => {
      if (typeof arg === "string") {
        return arg;
      } else if (typeof arg === "function") {
        return `[Function: ${arg.name || "anonymous"}]`;
      } else if (typeof arg === "undefined") {
        return "undefined";
      } else if (arg instanceof Error) {
        return arg.stack || arg.message || String(arg);
      } else if (typeof arg === "object" && arg !== null) {
        if (arg.constructor && arg.constructor.name) {
          return `[${arg.constructor.name}] ${JSON.stringify(arg)}`;
        } else {
          try {
            return JSON.stringify(arg);
          } catch (e) {
            return String(arg);
          }
        }
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
globalThis.console = globalThis.console ?? {
  trace: (...args: any[]) => log(0, ...args),
  debug: (...args: any[]) => log(1, ...args),
  info: (...args: any[]) => log(2, ...args),
  warn: (...args: any[]) => log(3, ...args),
  error: (...args: any[]) => log(4, ...args),
  log: (...args: any[]) => log(2, ...args),
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

async function renderTeX(
  math: string,
  fontSize: number,
  align: 0 | 1 | 2,
): Promise<string> {
  if (
    typeof fontSize !== "number" ||
    !Number.isFinite(fontSize) ||
    fontSize <= 0
  ) {
    throw new Error(`Font size must be positive and finite: ${fontSize}`);
  }

  svg.options.displayAlign =
    align === 0 ? "left" : align === 1 ? "center" : "right";
  const mathItem: LiteElement = await mathJax.convertPromise(math, {
    display: true,
    em: fontSize,
    ex: fontSize / 2,
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

globalThis.__entry_init = init;
globalThis.__entry_renderTeX = renderTeX;
