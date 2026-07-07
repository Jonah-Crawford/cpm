// lib/utils.js – shadcn/ui `cn` helper (vanilla JS)
function cn(...inputs) {
  return inputs
    .flat()
    .filter(Boolean)
    .map((x) => {
      if (typeof x === "string") return x;
      if (Array.isArray(x)) return cn(...x);
      if (typeof x === "object" && x !== null) {
        return Object.entries(x)
          .filter(([, v]) => v)
          .map(([k]) => k)
          .join(" ");
      }
      return "";
    })
    .join(" ");
}
