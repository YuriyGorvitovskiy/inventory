pub const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Inventory UI</title>
  <style>
    :root {
      --panel: #ffffff;
      --accent: #0f766e;
      --accent-2: #115e59;
      --text: #0f172a;
      --muted: #64748b;
      --border: #dbe2ea;
      --danger: #b91c1c;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
      color: var(--text);
      background: linear-gradient(120deg, #f7fafc 0%, #eef7f5 100%);
      min-height: 100vh;
      padding: 24px;
    }
    .wrap {
      max-width: 980px;
      margin: 0 auto;
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 16px;
      padding: 20px;
      box-shadow: 0 10px 28px rgba(15, 23, 42, 0.08);
    }
    .head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 14px;
      gap: 10px;
    }
    h1 {
      margin: 0 0 4px;
      font-size: 28px;
      letter-spacing: 0.2px;
    }
    .subtitle { margin: 0; color: var(--muted); }
    input {
      width: 100%;
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 8px 10px;
      font-size: 14px;
    }
    button {
      border: 0;
      border-radius: 10px;
      padding: 8px 12px;
      font-size: 14px;
      cursor: pointer;
      background: var(--accent);
      color: white;
      font-weight: 600;
      white-space: nowrap;
    }
    button:hover { background: var(--accent-2); }
    button.secondary {
      background: transparent;
      color: var(--accent);
      border: 1px solid var(--accent);
    }
    button.danger {
      background: white;
      color: var(--danger);
      border: 1px solid #fecaca;
    }
    button.ghost {
      background: white;
      color: var(--muted);
      border: 1px solid var(--border);
    }
    table {
      width: 100%;
      border-collapse: collapse;
      border: 1px solid var(--border);
      border-radius: 12px;
      overflow: hidden;
    }
    th, td {
      text-align: left;
      padding: 10px;
      border-bottom: 1px solid var(--border);
      vertical-align: middle;
    }
    th {
      background: #f8fafc;
      color: #334155;
      font-size: 13px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    tr:last-child td { border-bottom: 0; }
    .row-actions {
      display: flex;
      gap: 6px;
    }
    .muted { color: var(--muted); }
    .error {
      color: var(--danger);
      margin-top: 10px;
      min-height: 20px;
    }
    .id-cell { color: var(--muted); font-size: 12px; }
    .qty-input { max-width: 110px; }
  </style>
</head>
<body>
  <div class="wrap">
    <div class="head">
      <div>
        <h1>Household Inventory</h1>
        <p class="subtitle">Inline table editing wired to <code>/api/items</code></p>
      </div>
      <button id="add-row-btn" title="Add row">+ Add row</button>
    </div>

    <table>
      <thead>
        <tr>
          <th style="width: 60px;">ID</th>
          <th>Name</th>
          <th>Category</th>
          <th style="width: 140px;">Quantity</th>
          <th style="width: 250px;">Actions</th>
        </tr>
      </thead>
      <tbody id="items-body"></tbody>
    </table>
    <p id="empty" class="muted">No items yet. Click <b>+ Add row</b> to create one.</p>
    <p id="error" class="error"></p>
  </div>

  <script>
    const bodyEl = document.getElementById("items-body");
    const emptyEl = document.getElementById("empty");
    const errorEl = document.getElementById("error");
    const addRowBtn = document.getElementById("add-row-btn");

    let itemsCache = [];
    let draftRowActive = false;

    function setError(message) {
      errorEl.textContent = message || "";
    }

    function escapeHtml(value) {
      return String(value)
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll("\"", "&quot;")
        .replaceAll("'", "&#39;");
    }

    async function fetchItems() {
      const res = await fetch("/api/items");
      if (!res.ok) throw new Error("Failed to fetch items");
      return res.json();
    }

    async function createItem(payload) {
      const res = await fetch("/api/items", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(await res.text() || "Failed to create item");
    }

    async function updateItem(id, payload) {
      const res = await fetch(`/api/items/${id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(await res.text() || "Failed to update item");
    }

    async function removeItem(id) {
      const res = await fetch(`/api/items/${id}`, { method: "DELETE" });
      if (!res.ok) throw new Error(await res.text() || "Failed to delete item");
    }

    function validatePayload(payload) {
      if (!payload.name || !payload.category) {
        throw new Error("Name and category are required");
      }
      if (!Number.isInteger(payload.quantity) || payload.quantity < 0) {
        throw new Error("Quantity must be a non-negative integer");
      }
    }

    function rowEditorMarkup(id, initial) {
      return `
        <td class="id-cell">${id ? id : "new"}</td>
        <td><input data-field="name" value="${escapeHtml(initial.name || "")}" placeholder="Item name" /></td>
        <td><input data-field="category" value="${escapeHtml(initial.category || "")}" placeholder="Category" /></td>
        <td><input data-field="quantity" class="qty-input" type="number" min="0" value="${Number.isInteger(initial.quantity) ? initial.quantity : 0}" /></td>
        <td>
          <div class="row-actions">
            <button class="save-btn">${id ? "Save" : "Create"}</button>
            <button class="ghost cancel-btn">Cancel</button>
            ${id ? `<button class="danger delete-btn">Delete</button>` : ""}
          </div>
        </td>
      `;
    }

    function rowReadMarkup(item) {
      return `
        <td class="id-cell">${item.id}</td>
        <td>${escapeHtml(item.name)}</td>
        <td>${escapeHtml(item.category)}</td>
        <td>${item.quantity}</td>
        <td>
          <div class="row-actions">
            <button class="secondary edit-btn">Edit</button>
            <button class="danger delete-btn">Delete</button>
          </div>
        </td>
      `;
    }

    function getPayloadFromRow(tr) {
      return {
        name: tr.querySelector('[data-field="name"]').value.trim(),
        category: tr.querySelector('[data-field="category"]').value.trim(),
        quantity: Number(tr.querySelector('[data-field="quantity"]').value),
      };
    }

    function bindReadRow(tr, item) {
      tr.querySelector(".edit-btn").addEventListener("click", () => {
        tr.innerHTML = rowEditorMarkup(item.id, item);
        bindEditRow(tr, item.id, item);
      });

      tr.querySelector(".delete-btn").addEventListener("click", async () => {
        try {
          await removeItem(item.id);
          await refresh();
        } catch (err) {
          setError(err.message || "Delete failed");
        }
      });
    }

    function bindEditRow(tr, id, original) {
      tr.querySelector(".save-btn").addEventListener("click", async () => {
        try {
          setError("");
          const payload = getPayloadFromRow(tr);
          validatePayload(payload);

          if (id) {
            await updateItem(id, payload);
          } else {
            await createItem(payload);
            draftRowActive = false;
          }
          await refresh();
        } catch (err) {
          setError(err.message || "Save failed");
        }
      });

      tr.querySelector(".cancel-btn").addEventListener("click", async () => {
        if (!id) {
          draftRowActive = false;
          tr.remove();
          if (!bodyEl.children.length) {
            emptyEl.style.display = "block";
          }
          return;
        }
        tr.innerHTML = rowReadMarkup(original);
        bindReadRow(tr, original);
      });

      const deleteBtn = tr.querySelector(".delete-btn");
      if (deleteBtn) {
        deleteBtn.addEventListener("click", async () => {
          try {
            await removeItem(id);
            await refresh();
          } catch (err) {
            setError(err.message || "Delete failed");
          }
        });
      }
    }

    function renderRows(items) {
      bodyEl.innerHTML = "";
      emptyEl.style.display = items.length ? "none" : "block";

      for (const item of items) {
        const tr = document.createElement("tr");
        tr.innerHTML = rowReadMarkup(item);
        bodyEl.appendChild(tr);
        bindReadRow(tr, item);
      }
    }

    async function refresh() {
      setError("");
      itemsCache = await fetchItems();
      renderRows(itemsCache);
    }

    addRowBtn.addEventListener("click", () => {
      setError("");
      if (draftRowActive) {
        setError("Finish the new row first.");
        return;
      }
      draftRowActive = true;
      emptyEl.style.display = "none";

      const tr = document.createElement("tr");
      tr.innerHTML = rowEditorMarkup(null, { name: "", category: "", quantity: 0 });
      bodyEl.prepend(tr);
      bindEditRow(tr, null, null);
    });

    refresh().catch((err) => setError(err.message || "Failed to load"));
  </script>
</body>
</html>
"#;
