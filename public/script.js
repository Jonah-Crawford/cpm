async function loadPackages() {
  const container = document.getElementById("packages");

  try {
    const res = await fetch("/api/packages");

    if (!res.ok) {
      throw new Error(await res.text());
    }

    const packages = await res.json();

    if (packages.length === 0) {
      container.innerHTML = "<p>No packages published yet.</p>";
      return;
    }

    container.innerHTML = packages.map(pkg => `
      <article class="package-card">
        <h3>${escapeHtml(pkg.name)}</h3>
        <p>${escapeHtml(pkg.description || "No description provided.")}</p>
        <div class="meta">
          <span>Latest: ${escapeHtml(pkg.latest || "none")}</span>
          <span>Author: ${escapeHtml(pkg.author || "unknown")}</span>
        </div>
        <code>craw pkg install ${escapeHtml(pkg.name)}</code>
      </article>
    `).join("");
  } catch (err) {
    container.innerHTML = `<p class="error">Failed to load packages: ${escapeHtml(err.message)}</p>`;
  }
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

loadPackages();
