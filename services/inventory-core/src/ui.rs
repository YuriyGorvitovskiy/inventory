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
      --border-strong: #cbd5e1;
      --danger: #b91c1c;
      --danger-soft: #fee2e2;
      --surface: #f8fafc;
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
    .shell {
      max-width: 980px;
      margin: 0 auto;
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 16px;
      padding: 20px;
      box-shadow: 0 10px 28px rgba(15, 23, 42, 0.08);
    }
    .view-head {
      display: flex;
      align-items: end;
      justify-content: space-between;
      gap: 12px;
      margin-bottom: 18px;
    }
    h1 {
      margin: 0;
      font-size: 28px;
      letter-spacing: 0.2px;
    }
    .subtitle {
      margin: 6px 0 0;
      color: var(--muted);
    }
    .stack {
      display: grid;
      gap: 16px;
    }
    .action-bar {
      display: flex;
      justify-content: flex-end;
      gap: 8px;
      flex-wrap: wrap;
    }
    .panel {
      border: 1px solid var(--border);
      border-radius: 14px;
      background: var(--panel);
      overflow: hidden;
    }
    table {
      width: 100%;
      border-collapse: collapse;
    }
    th, td {
      text-align: left;
      padding: 10px;
      border-bottom: 1px solid var(--border);
      vertical-align: middle;
    }
    th {
      background: var(--surface);
      color: #334155;
      font-size: 13px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    tr:last-child td { border-bottom: 0; }
    .row-actions {
      display: flex;
      gap: 6px;
      flex-wrap: wrap;
      justify-content: flex-end;
    }
    .empty {
      margin: 0;
      padding: 18px;
      color: var(--muted);
    }
    .error {
      color: var(--danger);
      min-height: 20px;
      margin: 0;
    }
    .text-card {
      border: 1px solid var(--border);
      background: var(--surface);
      border-radius: 14px;
      padding: 14px 16px;
      color: var(--text);
    }
    .meta {
      color: var(--muted);
      font-size: 12px;
    }
    input {
      width: 100%;
      border: 1px solid var(--border-strong);
      border-radius: 10px;
      padding: 8px 10px;
      font-size: 14px;
      background: #fff;
    }
    input:focus {
      outline: 2px solid rgba(15, 118, 110, 0.15);
      border-color: var(--accent);
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
    button.ghost {
      background: white;
      color: var(--muted);
      border: 1px solid var(--border);
    }
    button.danger {
      background: white;
      color: var(--danger);
      border: 1px solid #fecaca;
    }
    @media (max-width: 720px) {
      body { padding: 12px; }
      .shell { padding: 16px; }
      .view-head { align-items: start; flex-direction: column; }
      th, td { padding: 8px; }
      .row-actions { justify-content: start; }
    }
  </style>
</head>
<body>
  <div class="shell">
    <div id="app" class="stack"></div>
    <p id="error" class="error"></p>
  </div>

  <script>
    const appEl = document.getElementById("app");
    const errorEl = document.getElementById("error");

    const state = {
      currentView: {
        name: "inventory.item.list",
        params: {},
      },
      view: null,
      draftRowActive: false,
      editingRowId: null,
    };

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

    function viewUrl(name, params) {
      const query = new URLSearchParams();
      for (const [key, value] of Object.entries(params || {})) {
        if (value == null) continue;
        query.set(key, String(value));
      }

      const base = name === "inventory.item.list" ? "/api/views/items" : `/api/views/${encodeURIComponent(name)}`;
      const suffix = query.toString();
      return suffix ? `${base}?${suffix}` : base;
    }

    async function fetchResolvedView(name, params) {
      const res = await fetch(viewUrl(name, params));
      if (!res.ok) {
        throw new Error(await res.text() || "Failed to fetch view");
      }
      return res.json();
    }

    async function executeAction(actionName, payload) {
      const res = await fetch(`/api/actions/${encodeURIComponent(actionName)}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!res.ok) throw new Error(await res.text() || `Failed to execute action ${actionName}`);
      return res.status === 204 ? null : res.json();
    }

    function validatePayload(payload) {
      if (!payload.name || !payload.category) {
        throw new Error("Name and category are required");
      }
      if (!Number.isInteger(payload.quantity) || payload.quantity < 0) {
        throw new Error("Quantity must be a non-negative integer");
      }
    }

    function interactionFor(eventName) {
      return state.view?.definition?.interactions?.find((entry) => entry.event === eventName) || null;
    }

    function columnsForTable(widget) {
      return widget.columns || [];
    }

    function isEditorView() {
      return state.currentView.name === "inventory.item.editor";
    }

    function navigateToView(name, params = {}) {
      state.currentView = { name, params };
      state.draftRowActive = false;
      state.editingRowId = null;
      return refresh();
    }

    function inputTypeFor(editorKind) {
      switch (editorKind) {
        case "integer":
        case "number":
        case "float":
          return "number";
        case "email":
          return "email";
        case "timestamp":
          return "datetime-local";
        default:
          return "text";
      }
    }

    function numericStepFor(editorKind) {
      return editorKind === "float" ? "any" : "1";
    }

    function editorMarkup(column, initialValue) {
      const inputType = inputTypeFor(column.editor_kind);
      const attrs = [`data-field="${escapeHtml(column.key)}"`];
      if (inputType === "number") {
        attrs.push(`step="${numericStepFor(column.editor_kind)}"`);
        if (column.key === "quantity") {
          attrs.push("min=\"0\"");
        }
      }

      const value = initialValue ?? "";
      return `<input type="${inputType}" ${attrs.join(" ")} value="${escapeHtml(value)}" />`;
    }

    function readCellMarkup(value, column) {
      if (value == null) {
        return "<span class=\"meta\">-</span>";
      }
      if (column.editor_kind === "integer" || column.editor_kind === "number" || column.editor_kind === "float") {
        return escapeHtml(value);
      }
      return escapeHtml(value);
    }

    function payloadFromRow(tr, columns) {
      const payload = {};
      for (const column of columns) {
        const input = tr.querySelector(`[data-field="${column.key}"]`);
        if (!input) continue;

        if (column.editor_kind === "integer" || column.editor_kind === "number") {
          payload[column.key] = Number(input.value);
        } else if (column.editor_kind === "float") {
          payload[column.key] = Number(input.value);
        } else {
          payload[column.key] = input.value.trim();
        }
      }
      return payload;
    }

    function actionButtonClass(actionName) {
      if (actionName.includes(".delete")) return "danger";
      if (actionName.includes(".update")) return "secondary";
      return "";
    }

    function actionLabel(action) {
      const tail = action.name.split(".").pop() || action.name;
      return tail
        .split("_")
        .join(" ")
        .replace(/\b\w/g, (match) => match.toUpperCase());
    }

    function rowIdentity(row) {
      return row?.source?.id ?? null;
    }

    function mappingValue(bind, scope = {}) {
      if (typeof bind !== "string") {
        return bind;
      }
      if (!bind.startsWith("$")) {
        return bind;
      }

      const segments = bind.slice(1).split(".");
      let current;
      if (segments[0] === "params") {
        current = state.view?.params || {};
      } else if (segments[0] === "context") {
        current = state.view?.context || {};
      } else if (segments[0] === "row") {
        current = scope.row || null;
      } else {
        return undefined;
      }

      for (const segment of segments.slice(1)) {
        if (current == null) return undefined;
        current = current[segment];
      }
      return current;
    }

    function resolveInteractionParams(interaction, scope = {}) {
      const resolved = {};
      for (const param of interaction?.params || []) {
        resolved[param.name] = mappingValue(param.value?.bind, scope);
      }
      return resolved;
    }

    function renderActionButtons(actions) {
      return actions.map((action) => {
        const cls = actionButtonClass(action.name);
        const title = escapeHtml(action.description || action.name);
        return `<button data-action="${escapeHtml(action.name)}" class="${cls}" title="${title}">${escapeHtml(actionLabel(action))}</button>`;
      }).join("");
    }

    function routeLabel(eventName) {
      if (eventName === "back") return "Back";
      const normalized = eventName.replaceAll("_", " ");
      return normalized.charAt(0).toUpperCase() + normalized.slice(1);
    }

    function renderRouteButtons(interactions, scope = {}) {
      return interactions
        .filter((interaction) => interaction?.route_to)
        .map((interaction) => {
          const params = resolveInteractionParams(interaction, scope);
          const paramsAttr = escapeHtml(JSON.stringify(params));
          return `<button data-route-to="${escapeHtml(interaction.route_to)}" data-route-event="${escapeHtml(interaction.event)}" data-route-params="${paramsAttr}" class="secondary">${escapeHtml(routeLabel(interaction.event))}</button>`;
        })
        .join("");
    }

    function renderDisplayRow(row, widget) {
      const rowId = rowIdentity(row);
      const cols = columnsForTable(widget)
        .map((column) => `<td>${readCellMarkup(row.cells[column.key], column)}</td>`)
        .join("");

      const actionInteractions = [];
      const routeInteractions = [];
      const updateInteraction = interactionFor("update");
      const deleteInteraction = interactionFor("delete");
      if (updateInteraction?.action) {
        actionInteractions.push({ name: updateInteraction.action, description: "Update this row" });
      }
      if (updateInteraction?.route_to) {
        routeInteractions.push(updateInteraction);
      }
      if (deleteInteraction?.action) {
        actionInteractions.push({ name: deleteInteraction.action, description: "Delete this row" });
      }
      if (deleteInteraction?.route_to) {
        routeInteractions.push(deleteInteraction);
      }

      return `
        <tr data-row-id="${rowId == null ? "" : escapeHtml(rowId)}">
          ${cols}
          <td>
            <div class="row-actions">
              ${renderRouteButtons(routeInteractions, { row: row.source })}
              ${renderActionButtons(actionInteractions)}
            </div>
          </td>
        </tr>
      `;
    }

    function renderEditRow(row, widget, isCreate) {
      const source = row?.source || {};
      const cells = columnsForTable(widget)
        .map((column) => {
          const editable = isCreate || column.editable;
          const value = row?.cells?.[column.key] ?? source[column.key] ?? "";
          return `<td>${editable ? editorMarkup(column, value) : readCellMarkup(value, column)}</td>`;
        })
        .join("");

      return `
        <tr data-draft="${isCreate ? "true" : "false"}" data-row-id="${source.id == null ? "" : escapeHtml(source.id)}">
          ${cells}
          <td>
            <div class="row-actions">
              <button data-row-save="true">${isCreate ? "Create" : "Save"}</button>
              <button data-row-cancel="true" class="ghost">Cancel</button>
              ${!isCreate && interactionFor("delete")?.action ? `<button data-action="${escapeHtml(interactionFor("delete").action)}" class="danger">Delete</button>` : ""}
            </div>
          </td>
        </tr>
      `;
    }

    function renderTable(widget) {
      const headers = columnsForTable(widget)
        .map((column) => `<th>${escapeHtml(column.header)}</th>`)
        .join("");

      const rows = widget.rows || [];
      const body = rows.length
        ? rows
            .map((row) => {
              const isEditing = state.editingRowId !== null && rowIdentity(row) === state.editingRowId;
              return isEditing ? renderEditRow(row, widget, false) : renderDisplayRow(row, widget);
            })
            .join("")
        : `<tr><td colspan="${columnsForTable(widget).length + 1}" class="empty">No items yet. Use the create action to add one.</td></tr>`;

      const draft = state.draftRowActive ? renderEditRow(null, widget, true) : "";

      return `
        <section class="panel">
          <table>
            <thead>
              <tr>
                ${headers}
                <th style="width: 220px;">Actions</th>
              </tr>
            </thead>
            <tbody>
              ${draft}
              ${body}
            </tbody>
          </table>
        </section>
      `;
    }

    function renderText(widget) {
      return `<section class="text-card">${escapeHtml(widget.text || "")}</section>`;
    }

    function renderForm(widget) {
      const fieldMarkup = (widget.fields || [])
        .map((field) => `
          <label class="stack" style="gap: 6px;">
            <span>${escapeHtml(field.label)}${field.required ? " *" : ""}</span>
            ${field.editable ? `<input data-form-field="${escapeHtml(field.key)}" type="${inputTypeFor(field.editor_kind)}" ${inputTypeFor(field.editor_kind) === "number" ? `step="${numericStepFor(field.editor_kind)}"` : ""} value="${escapeHtml(field.value ?? "")}" />` : `<div class="text-card">${escapeHtml(field.value ?? "")}</div>`}
          </label>
        `)
        .join("");

      const saveInteraction = interactionFor("save");
      const deleteInteraction = interactionFor("delete");
      const backInteraction = interactionFor("back");

      return `
        <section class="panel" style="padding: 18px;">
          <div class="stack">
            ${fieldMarkup}
            <div class="row-actions">
              ${backInteraction?.route_to ? renderRouteButtons([backInteraction]) : ""}
              ${deleteInteraction?.action ? `<button data-form-action="${escapeHtml(deleteInteraction.action)}" class="danger">Delete</button>` : ""}
              ${saveInteraction?.action ? `<button data-form-save="${escapeHtml(saveInteraction.action)}">Save</button>` : ""}
            </div>
          </div>
        </section>
      `;
    }

    function renderActionBar(widget) {
      const backInteraction = interactionFor("back");
      return `<section class="action-bar">${backInteraction?.route_to ? renderRouteButtons([backInteraction]) : ""}${renderActionButtons(widget.actions || [])}</section>`;
    }

    function renderWidget(widget) {
      if (!widget) return "";

      switch (widget.type) {
        case "page": {
          const children = (widget.children || []).map(renderWidget).join("");
          const description = escapeHtml(state.view?.definition?.name || "");
          return `
            <section class="stack">
              <header class="view-head">
                <div>
                  <h1>${escapeHtml(widget.title || "Inventory")}</h1>
                  <p class="subtitle">Rendered from <code>${escapeHtml(viewUrl(state.currentView.name, state.currentView.params))}</code> using the server-resolved widget tree.</p>
                </div>
                <span class="meta">${description}</span>
              </header>
              ${children}
            </section>
          `;
        }
        case "action_bar":
          return renderActionBar(widget);
        case "table":
          return renderTable(widget);
        case "text":
          return renderText(widget);
        case "form":
          return renderForm(widget);
        default:
          return `<section class="text-card">Unsupported widget type: ${escapeHtml(widget.type || "unknown")}</section>`;
      }
    }

    function bindActionHandlers() {
      appEl.querySelectorAll("[data-action]").forEach((button) => {
        button.addEventListener("click", async () => {
          const actionName = button.getAttribute("data-action");
          try {
            setError("");

            if (actionName === interactionFor("create")?.action) {
              if (state.draftRowActive) {
                throw new Error("Finish the new row first.");
              }
              state.draftRowActive = true;
              state.editingRowId = null;
              render();
              return;
            }

            const tr = button.closest("tr");
            const rowId = Number(tr?.getAttribute("data-row-id"));
            if (!Number.isInteger(rowId)) {
              throw new Error("Missing row id");
            }

            if (actionName === interactionFor("update")?.action) {
              state.draftRowActive = false;
              state.editingRowId = rowId;
              render();
              return;
            }

            if (actionName === interactionFor("save")?.action) {
              state.editingRowId = rowId;
              render();
              return;
            }

            if (actionName === interactionFor("delete")?.action) {
              await executeAction(actionName, { target_id: rowId, fields: {} });
              if (isEditorView()) {
                await navigateToView("inventory.item.list", {});
              } else {
                state.editingRowId = null;
                await refresh();
              }
              return;
            }

            throw new Error(`Unsupported action '${actionName}'`);
          } catch (err) {
            setError(err.message || "Action failed");
          }
        });
      });

      appEl.querySelectorAll("[data-route-to]").forEach((button) => {
        button.addEventListener("click", async () => {
          try {
            setError("");
            const routeTo = button.getAttribute("data-route-to");
            const params = JSON.parse(button.getAttribute("data-route-params") || "{}");
            await navigateToView(routeTo, params);
          } catch (err) {
            setError(err.message || "Navigation failed");
          }
        });
      });

      appEl.querySelectorAll("[data-row-save]").forEach((button) => {
        button.addEventListener("click", async () => {
          const tr = button.closest("tr");
          const widget = state.view?.widget?.children?.find((child) => child.type === "table");
          const columns = columnsForTable(widget || {});

          try {
            setError("");
            const payload = payloadFromRow(tr, columns);
            validatePayload(payload);

            const rowIdValue = tr.getAttribute("data-row-id");
            const rowId = rowIdValue ? Number(rowIdValue) : null;
            if (tr.dataset.draft === "true") {
              const createAction = interactionFor("create")?.action;
              if (!createAction) {
                throw new Error("Create interaction is not configured");
              }
              await executeAction(createAction, { fields: payload });
              state.draftRowActive = false;
            } else {
              const updateAction = interactionFor("update")?.action;
              const saveAction = interactionFor("save")?.action;
              const actionName = saveAction || updateAction;
              if (!actionName) {
                throw new Error("Save interaction is not configured");
              }
              await executeAction(actionName, { target_id: rowId, fields: payload });
              state.editingRowId = null;
            }

            if (isEditorView()) {
              await navigateToView("inventory.item.editor", { id: rowId });
            } else {
              await refresh();
            }
          } catch (err) {
            setError(err.message || "Save failed");
          }
        });
      });

      appEl.querySelectorAll("[data-row-cancel]").forEach((button) => {
        button.addEventListener("click", () => {
          state.draftRowActive = false;
          state.editingRowId = null;
          setError("");
          render();
        });
      });

      appEl.querySelectorAll("[data-form-save]").forEach((button) => {
        button.addEventListener("click", async () => {
          try {
            setError("");
            const payload = {};
            appEl.querySelectorAll("[data-form-field]").forEach((input) => {
              const key = input.getAttribute("data-form-field");
              if (input.type === "number") {
                payload[key] = Number(input.value);
              } else {
                payload[key] = input.value.trim();
              }
            });
            validatePayload(payload);

            const actionName = button.getAttribute("data-form-save");
            const targetId = Number(state.currentView.params.id);
            await executeAction(actionName, { target_id: targetId, fields: payload });
            await navigateToView(state.currentView.name, state.currentView.params);
          } catch (err) {
            setError(err.message || "Save failed");
          }
        });
      });

      appEl.querySelectorAll("[data-form-action]").forEach((button) => {
        button.addEventListener("click", async () => {
          try {
            setError("");
            const actionName = button.getAttribute("data-form-action");
            const targetId = Number(state.currentView.params.id);
            await executeAction(actionName, { target_id: targetId, fields: {} });
            await navigateToView("inventory.item.list", {});
          } catch (err) {
            setError(err.message || "Action failed");
          }
        });
      });
    }

    function render() {
      appEl.innerHTML = renderWidget(state.view?.widget);
      bindActionHandlers();
    }

    async function refresh() {
      setError("");
      state.view = await fetchResolvedView(state.currentView.name, state.currentView.params);
      render();
    }

    refresh().catch((err) => setError(err.message || "Failed to load"));
  </script>
</body>
</html>
"#;
