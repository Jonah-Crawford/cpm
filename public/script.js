async function loadPackages() {
  const container = document.querySelector("div.packages");

  try {
    // const res = await fetch("https://cpm.ultimatecraw.xyz/api/packages");
    //
    // if (!res.ok) {
    //   throw new Error(await res.text());
    // }
    //
    const packages = [
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
      {
        name: "graphics",
        description: "graphics, a Crawssembly package.",
        author: "The Craw",
        latest: "1.0.1",
        published_at: "2026-07-07 14:49:52",
      },
    ];

    if (packages.length === 0) {
      container.innerHTML = "<p>No packages published yet.</p>";
      return;
    }

    container.innerHTML = packages
      .map(
        (pkg) => `
          <article class="package_card">
            <h3>${escapeHtml(pkg.name)}</h3>
            <p>${escapeHtml(pkg.description || "<i>No description provided.</i>")}</p>
            <div class="meta">
              <span>Latest: ${escapeHtml(pkg.latest || "none")}</span>
              <span>Author: ${escapeHtml(pkg.author || "unknown")}</span>
            </div>
            <code>craw pkg install ${escapeHtml(pkg.name)}</code>
          </article>
        `,
      )
      .join("");
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

document.addEventListener("DOMContentLoaded", loadPackages());
