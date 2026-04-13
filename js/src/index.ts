import { mathjax } from "@mathjax/src/js/mathjax.js";
import { TeX } from "@mathjax/src/js/input/tex.js";
import { liteAdaptor } from "@mathjax/src/js/adaptors/liteAdaptor.js";
import { RegisterHTMLHandler } from "@mathjax/src/js/handlers/html.js";
import { SVG } from "@mathjax/src/js/output/svg.js";
import { LiteElement } from "@mathjax/src/js/adaptors/lite/Element.js";

import "@mathjax/src/js/input/tex/base/BaseConfiguration.js";
import "@mathjax/src/js/input/tex/ams/AmsConfiguration.js";
import "@mathjax/src/js/input/tex/newcommand/NewcommandConfiguration.js";
import "@mathjax/src/js/input/tex/noundefined/NoUndefinedConfiguration.js";

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

function renderTeX(math: string): string {
  const mathItem: LiteElement = mathJax.convert(math, {
    display: true,
  });
  return adaptor.innerHTML(mathItem);
}

globalThis.__entry_renderTeX = renderTeX;
