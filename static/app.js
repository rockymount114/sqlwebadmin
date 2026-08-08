// SQL Web Admin - JavaScript Application Logic

document.addEventListener('DOMContentLoaded', async () => {
    // --- Application State ---
    let appState = {
        driver: 'mssql',
        database: 'master',
        allDatabases: false,
        connectionString: '',
        editor: null,
        history: JSON.parse(localStorage.getItem('sqlwebadmin_history') || '[]'),
        activeResults: null,
    };

    // --- DOM Elements ---
    const dbDriverSelect = document.getElementById('dbDriverSelect');
    const dbNameSelect = document.getElementById('dbNameSelect');
    const toggleAllDbsBtn = document.getElementById('toggleAllDbsBtn');
    const connectionBtn = document.getElementById('connectionBtn');
    const connStatus = document.getElementById('connStatus');
    const treeView = document.getElementById('treeView');
    const refreshTreeBtn = document.getElementById('refreshTreeBtn');
    const schemaSearchInput = document.getElementById('schemaSearchInput');
    const queryTextarea = document.getElementById('queryTextarea');
    const executeBtn = document.getElementById('executeBtn');
    const exportCsvBtn = document.getElementById('exportCsvBtn');
    const resetBtn = document.getElementById('resetBtn');
    const formatSqlBtn = document.getElementById('formatSqlBtn');
    const copyQueryBtn = document.getElementById('copyQueryBtn');
    const statusBar = document.getElementById('statusBar');
    const returnLabel = document.getElementById('returnLabel');
    const timeLabel = document.getElementById('timeLabel');
    const errorLabel = document.getElementById('errorLabel');
    const resultsContainer = document.getElementById('resultsContainer');
    const emptyResultsState = document.getElementById('emptyResultsState');
    const tableFilterInput = document.getElementById('tableFilterInput');
    const tableSearchBox = document.getElementById('tableSearchBox');
    const historyList = document.getElementById('historyList');
    
    // Modal Elements
    const connectionModal = document.getElementById('connectionModal');
    const closeModalBtn = document.getElementById('closeModalBtn');
    const modalDriverSelect = document.getElementById('modalDriverSelect');
    const connectionStringInput = document.getElementById('connectionStringInput');
    const testConnectionBtn = document.getElementById('testConnectionBtn');
    const saveConnectionBtn = document.getElementById('saveConnectionBtn');
    const modalTestResult = document.getElementById('modalTestResult');

    // --- Helper to parse JSON safely ---
    async function safeFetchJson(url, options = {}) {
        const res = await fetch(url, options);
        const contentType = res.headers.get('content-type') || '';
        if (contentType.includes('application/json')) {
            return await res.json();
        } else {
            const text = await res.text();
            return {
                success: false,
                error: text || `HTTP ${res.status} ${res.statusText}`,
                message: text || `HTTP ${res.status}`
            };
        }
    }

    // --- Initialize CodeMirror Editor ---
    appState.editor = CodeMirror.fromTextArea(queryTextarea, {
        mode: 'text/x-sql',
        theme: 'dracula',
        lineNumbers: true,
        indentUnit: 4,
        tabSize: 4,
        lineWrapping: true,
        extraKeys: {
            "Ctrl-Enter": () => executeQuery(),
            "Cmd-Enter": () => executeQuery(),
        }
    });

    appState.editor.setValue("-- Select a table from the Schema Explorer or enter your SQL query here\nSELECT * FROM sys.databases;");

    // --- Fetch Server Config ---
    try {
        const configData = await safeFetchJson('/api/config');
        if (configData.default_driver) {
            const savedConn = localStorage.getItem('sqlwebadmin_conn_' + configData.default_driver);
            appState.driver = configData.default_driver;
            appState.connectionString = savedConn || configData.default_connection_string;
            
            dbDriverSelect.value = appState.driver;
            modalDriverSelect.value = appState.driver;
            connectionStringInput.value = appState.connectionString;

            updateConnectionBadge(true, "Connected");
            await loadDatabasesList();
            loadSchemaTree();
        } else {
            updateConnectionBadge(false, "Config Error");
        }
    } catch (err) {
        console.error("Failed to load server config", err);
        updateConnectionBadge(false, "Disconnected");
    }

    // --- Event Listeners ---
    dbDriverSelect.addEventListener('change', async (e) => {
        appState.driver = e.target.value;
        const saved = localStorage.getItem('sqlwebadmin_conn_' + appState.driver);
        if (saved) appState.connectionString = saved;
        modalDriverSelect.value = appState.driver;
        connectionStringInput.value = appState.connectionString;
        await loadDatabasesList();
        loadSchemaTree();
    });

    dbNameSelect.addEventListener('change', (e) => {
        appState.database = e.target.value;
        loadSchemaTree();
    });

    toggleAllDbsBtn.addEventListener('click', () => {
        appState.allDatabases = !appState.allDatabases;
        toggleAllDbsBtn.style.color = appState.allDatabases ? 'var(--primary)' : 'var(--text-muted)';
        loadSchemaTree();
    });

    refreshTreeBtn.addEventListener('click', async () => {
        await loadDatabasesList();
        loadSchemaTree();
    });

    executeBtn.addEventListener('click', () => executeQuery());
    exportCsvBtn.addEventListener('click', () => exportQuery('csv'));
    resetBtn.addEventListener('click', () => resetWorkspace());
    copyQueryBtn.addEventListener('click', () => copyQueryToClipboard());
    formatSqlBtn.addEventListener('click', () => formatQuerySql());

    // Schema Search Filter
    schemaSearchInput.addEventListener('input', (e) => {
        const filter = e.target.value.toLowerCase();
        const nodes = treeView.querySelectorAll('.tree-node');
        nodes.forEach(node => {
            const text = node.querySelector('.tree-node-text')?.textContent.toLowerCase() || '';
            if (text.includes(filter)) {
                node.style.display = 'block';
            } else {
                node.style.display = 'none';
            }
        });
    });

    // Table Row Search Filter
    tableFilterInput.addEventListener('input', (e) => {
        const filter = e.target.value.toLowerCase();
        const tables = resultsContainer.querySelectorAll('.data-table');
        tables.forEach(table => {
            const rows = table.querySelectorAll('tbody tr');
            rows.forEach(row => {
                const text = row.textContent.toLowerCase();
                row.style.display = text.includes(filter) ? '' : 'none';
            });
        });
    });

    // Tab Navigation
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const targetId = btn.getAttribute('data-target');
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(targetId).classList.add('active');
            
            if (targetId === 'historyResults') {
                renderHistoryList();
            }
        });
    });

    // Connection Modal Listeners
    connectionBtn.addEventListener('click', () => {
        modalDriverSelect.value = appState.driver;
        connectionStringInput.value = appState.connectionString;
        modalTestResult.style.display = 'none';
        connectionModal.classList.add('active');
    });

    closeModalBtn.addEventListener('click', () => connectionModal.classList.remove('active'));
    connectionModal.addEventListener('click', (e) => {
        if (e.target === connectionModal) connectionModal.classList.remove('active');
    });

    testConnectionBtn.addEventListener('click', async () => {
        const driver = modalDriverSelect.value;
        const connStr = connectionStringInput.value;
        modalTestResult.style.display = 'block';
        modalTestResult.className = 'modal-alert alert-info';
        modalTestResult.textContent = 'Testing connection...';

        try {
            const data = await safeFetchJson('/api/connect/test', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ driver, connection_string: connStr })
            });
            if (data.success) {
                modalTestResult.className = 'modal-alert alert-success';
                modalTestResult.textContent = data.message;
            } else {
                modalTestResult.className = 'modal-alert alert-error';
                modalTestResult.textContent = data.message || data.error;
            }
        } catch (err) {
            modalTestResult.className = 'modal-alert alert-error';
            modalTestResult.textContent = 'Error testing connection: ' + err.message;
        }
    });

    saveConnectionBtn.addEventListener('click', async () => {
        appState.driver = modalDriverSelect.value;
        appState.connectionString = connectionStringInput.value;
        dbDriverSelect.value = appState.driver;
        localStorage.setItem('sqlwebadmin_conn_' + appState.driver, appState.connectionString);
        connectionModal.classList.remove('active');
        updateConnectionBadge(true, "Connected");
        await loadDatabasesList();
        loadSchemaTree();
    });

    // --- Functions ---

    function updateConnectionBadge(connected, text) {
        const dot = connStatus.querySelector('.status-dot');
        const statusText = connStatus.querySelector('.status-text');
        if (connected) {
            dot.className = 'status-dot connected';
            statusText.textContent = text;
        } else {
            dot.className = 'status-dot error';
            statusText.textContent = text;
        }
    }

    // Load Available Databases List for Header Dropdown
    async function loadDatabasesList() {
        try {
            const data = await safeFetchJson(`/api/schema/databases?driver=${encodeURIComponent(appState.driver)}&connection_string=${encodeURIComponent(appState.connectionString)}`);
            if (data.success && data.databases && data.databases.length > 0) {
                dbNameSelect.innerHTML = data.databases.map(db => `<option value="${escapeHtml(db)}" ${db === appState.database ? 'selected' : ''}>${escapeHtml(db)}</option>`).join('');
                if (!data.databases.includes(appState.database)) {
                    appState.database = data.databases[0];
                    dbNameSelect.value = appState.database;
                }
            }
        } catch (err) {
            console.error("Failed to load databases list", err);
        }
    }

    // Load Schema Tree
    async function loadSchemaTree() {
        treeView.innerHTML = '<div class="loading-spinner"><i class="fa-solid fa-circle-notch fa-spin"></i> Loading schema...</div>';
        try {
            const data = await safeFetchJson(`/api/schema/tree?driver=${encodeURIComponent(appState.driver)}&connection_string=${encodeURIComponent(appState.connectionString)}&database=${encodeURIComponent(appState.database)}&all_databases=${appState.allDatabases}`);
            if (data.success && data.nodes) {
                treeView.innerHTML = '';
                data.nodes.forEach(node => {
                    treeView.appendChild(createTreeNode(node));
                });
                updateConnectionBadge(true, "Connected");
            } else {
                treeView.innerHTML = `<div class="alert alert-error">${escapeHtml(data.error || 'Failed to load schema tree')}</div>`;
                updateConnectionBadge(false, "Connection Error");
            }
        } catch (err) {
            treeView.innerHTML = `<div class="alert alert-error">Network error: ${escapeHtml(err.message)}</div>`;
            updateConnectionBadge(false, "Offline");
        }
    }

    // Create Tree Node Element
    function createTreeNode(nodeData) {
        const nodeEl = document.createElement('div');
        nodeEl.className = 'tree-node';
        nodeEl.dataset.id = nodeData.id;
        nodeEl.dataset.type = nodeData.node_type;
        nodeEl.dataset.value = nodeData.value;
        nodeEl.dataset.hasChildren = nodeData.has_children;

        const contentEl = document.createElement('div');
        contentEl.className = 'tree-content';

        let iconClass = 'fa-folder';
        if (nodeData.node_type === 'DATABASE') iconClass = 'fa-database db';
        else if (nodeData.node_type === 'TABLE' || nodeData.node_type === 'TABLE_ITEM' || nodeData.node_type === 'TABLE_GROUP') iconClass = 'fa-table table';
        else if (nodeData.node_type === 'VIEW' || nodeData.node_type === 'VIEW_ITEM' || nodeData.node_type === 'VIEW_GROUP') iconClass = 'fa-eye view';
        else if (nodeData.node_type === 'SPROC' || nodeData.node_type === 'SPROC_ITEM' || nodeData.node_type === 'SPROC_GROUP') iconClass = 'fa-gears sproc';
        else if (nodeData.node_type === 'COLUMN') iconClass = 'fa-columns column';
        else if (nodeData.node_type === 'PARAMETER') iconClass = 'fa-code-parameter param';

        contentEl.innerHTML = `
            ${nodeData.has_children ? '<span class="tree-toggle"><i class="fa-solid fa-chevron-right"></i></span>' : '<span style="width:14px"></span>'}
            <i class="fa-solid ${iconClass} tree-node-icon"></i>
            <span class="tree-node-text">${escapeHtml(nodeData.text)}</span>
        `;

        const childrenContainer = document.createElement('div');
        childrenContainer.className = 'tree-children';

        nodeEl.appendChild(contentEl);
        nodeEl.appendChild(childrenContainer);

        // Click Handler for Expansion & Action
        contentEl.addEventListener('click', async (e) => {
            e.stopPropagation();

            document.querySelectorAll('.tree-content').forEach(c => c.classList.remove('selected'));
            contentEl.classList.add('selected');

            // Handle Node Selection / Action
            if (nodeData.node_type === 'TABLE_ITEM' || nodeData.node_type === 'VIEW_ITEM' || nodeData.node_type === 'SPROC_ITEM') {
                loadObjectDefinition(nodeData.node_type, nodeData.id);
            }

            // Expand / Collapse Children
            if (nodeData.has_children) {
                const isOpen = nodeEl.classList.contains('open');
                if (isOpen) {
                    nodeEl.classList.remove('open');
                } else {
                    nodeEl.classList.add('open');
                    if (childrenContainer.children.length === 0) {
                        childrenContainer.innerHTML = '<div style="padding:4px 8px; color:var(--text-dim);"><i class="fa-solid fa-spinner fa-spin"></i> Loading...</div>';
                        try {
                            const data = await safeFetchJson(`/api/schema/children?node_type=${encodeURIComponent(nodeData.node_type)}&parent_id=${encodeURIComponent(nodeData.id)}&driver=${encodeURIComponent(appState.driver)}&connection_string=${encodeURIComponent(appState.connectionString)}&database=${encodeURIComponent(appState.database)}`);
                            childrenContainer.innerHTML = '';
                            if (data.success && data.nodes && data.nodes.length > 0) {
                                data.nodes.forEach(child => {
                                    childrenContainer.appendChild(createTreeNode(child));
                                });
                            } else {
                                childrenContainer.innerHTML = '<div style="padding:4px 8px; color:var(--text-dim);">No items found</div>';
                            }
                        } catch (err) {
                            childrenContainer.innerHTML = `<div style="padding:4px 8px; color:var(--error);">Failed to load</div>`;
                        }
                    }
                }
            }
        });

        return nodeEl;
    }

    // Load Definition or SELECT query for clicked node
    async function loadObjectDefinition(nodeType, objectId) {
        try {
            const data = await safeFetchJson(`/api/schema/definition?node_type=${encodeURIComponent(nodeType)}&object_id=${encodeURIComponent(objectId)}&driver=${encodeURIComponent(appState.driver)}&connection_string=${encodeURIComponent(appState.connectionString)}&database=${encodeURIComponent(appState.database)}`);
            if (data.success) {
                appState.editor.setValue(data.definition);
            }
        } catch (err) {
            console.error("Failed to load object definition", err);
        }
    }

    // Execute Query
    async function executeQuery() {
        const queryText = appState.editor.getValue().trim();
        if (!queryText) {
            showError("Query cannot be blank");
            return;
        }

        clearStatus();
        resultsContainer.innerHTML = '<div class="loading-spinner" style="padding: 40px; text-align:center;"><i class="fa-solid fa-circle-notch fa-spin fa-2x"></i><p style="margin-top:10px;">Executing query...</p></div>';
        emptyResultsState.style.display = 'none';

        const startTime = Date.now();

        try {
            const data = await safeFetchJson('/api/query/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    query: queryText,
                    driver: appState.driver,
                    connection_string: appState.connectionString,
                    database: appState.database
                })
            });
            const elapsed = Date.now() - startTime;

            addHistoryItem(queryText, data.success, data.execution_time_ms || elapsed);

            if (!data.success) {
                showError(data.error || "Query execution failed.");
                resultsContainer.innerHTML = '';
                emptyResultsState.style.display = 'flex';
                return;
            }

            statusBar.style.display = 'flex';
            errorLabel.style.display = 'none';
            returnLabel.textContent = `(${data.total_affected_rows} row(s) affected)`;
            timeLabel.textContent = `⚡ ${data.execution_time_ms} ms`;

            renderResults(data.tables);
        } catch (err) {
            showError("Execution error: " + err.message);
            resultsContainer.innerHTML = '';
            emptyResultsState.style.display = 'flex';
        }
    }

    // Render Query Result Data Tables
    function renderResults(tables) {
        resultsContainer.innerHTML = '';

        if (!tables || tables.length === 0) {
            emptyResultsState.style.display = 'flex';
            tableSearchBox.style.display = 'none';
            return;
        }

        emptyResultsState.style.display = 'none';
        tableSearchBox.style.display = 'block';

        tables.forEach((table, idx) => {
            const wrapper = document.createElement('div');
            wrapper.className = 'table-wrapper';

            if (tables.length > 1) {
                const title = document.createElement('div');
                title.style.padding = '8px 12px';
                title.style.fontWeight = '600';
                title.style.color = 'var(--primary)';
                title.textContent = `Result Set #${idx + 1} (${table.rows.length} rows)`;
                wrapper.appendChild(title);
            }

            const tbl = document.createElement('table');
            tbl.className = 'data-table';

            const thead = document.createElement('thead');
            const headerRow = document.createElement('tr');
            headerRow.innerHTML = '<th class="row-num">#</th>' + table.columns.map(c => `<th>${escapeHtml(c)}</th>`).join('');
            thead.appendChild(headerRow);
            tbl.appendChild(thead);

            const tbody = document.createElement('tbody');
            table.rows.forEach((row, rIdx) => {
                const tr = document.createElement('tr');
                let rowHtml = `<td class="row-num">${rIdx + 1}</td>`;
                row.forEach(val => {
                    if (val === null || val === undefined) {
                        rowHtml += '<td><span class="null-value">NULL</span></td>';
                    } else if (typeof val === 'object') {
                        rowHtml += `<td>${escapeHtml(JSON.stringify(val))}</td>`;
                    } else {
                        rowHtml += `<td>${escapeHtml(String(val))}</td>`;
                    }
                });
                tr.innerHTML = rowHtml;
                tbody.appendChild(tr);
            });
            tbl.appendChild(tbody);
            wrapper.appendChild(tbl);
            resultsContainer.appendChild(wrapper);
        });
    }

    // Export CSV
    async function exportQuery(format) {
        const queryText = appState.editor.getValue().trim();
        if (!queryText) {
            showError("Query cannot be blank");
            return;
        }

        try {
            const response = await fetch('/api/query/export', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    query: queryText,
                    driver: appState.driver,
                    connection_string: appState.connectionString,
                    database: appState.database
                })
            });

            if (!response.ok) {
                const errText = await response.text();
                showError("Export failed: " + errText);
                return;
            }

            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.style.display = 'none';
            a.href = url;
            a.download = 'query_export.csv';
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
        } catch (err) {
            showError("Export network error: " + err.message);
        }
    }

    // Helpers
    function showError(msg) {
        statusBar.style.display = 'flex';
        errorLabel.style.display = 'block';
        errorLabel.textContent = msg;
        returnLabel.textContent = '';
        timeLabel.textContent = '';
    }

    function clearStatus() {
        statusBar.style.display = 'none';
        errorLabel.style.display = 'none';
        errorLabel.textContent = '';
    }

    function resetWorkspace() {
        appState.editor.setValue('');
        clearStatus();
        resultsContainer.innerHTML = '';
        emptyResultsState.style.display = 'flex';
        tableSearchBox.style.display = 'none';
    }

    function copyQueryToClipboard() {
        const query = appState.editor.getValue();
        navigator.clipboard.writeText(query);
        copyQueryBtn.innerHTML = '<i class="fa-solid fa-check"></i> Copied!';
        setTimeout(() => {
            copyQueryBtn.innerHTML = '<i class="fa-regular fa-copy"></i> Copy';
        }, 1500);
    }

    function formatQuerySql() {
        let sql = appState.editor.getValue();
        const keywords = ['SELECT', 'FROM', 'WHERE', 'JOIN', 'LEFT JOIN', 'RIGHT JOIN', 'INNER JOIN', 'ON', 'GROUP BY', 'ORDER BY', 'HAVING', 'LIMIT', 'TOP', 'INSERT INTO', 'UPDATE', 'DELETE FROM', 'VALUES', 'AND', 'OR', 'IN', 'IS NULL', 'IS NOT NULL', 'AS'];
        keywords.forEach(kw => {
            const regex = new RegExp(`\\b${kw}\\b`, 'gi');
            sql = sql.replace(regex, kw);
        });
        appState.editor.setValue(sql);
    }

    function addHistoryItem(query, success, timeMs) {
        const item = {
            query,
            success,
            timeMs,
            timestamp: new Date().toLocaleTimeString()
        };
        appState.history.unshift(item);
        if (appState.history.length > 50) appState.history.pop();
        localStorage.setItem('sqlwebadmin_history', JSON.stringify(appState.history));
    }

    function renderHistoryList() {
        if (!appState.history || appState.history.length === 0) {
            historyList.innerHTML = '<div class="empty-state"><i class="fa-solid fa-history empty-icon"></i><p>No executed query history yet.</p></div>';
            return;
        }

        historyList.innerHTML = appState.history.map(item => `
            <div class="history-item" onclick="loadHistoryQuery(\`${escapeHtml(item.query).replace(/`/g, '\\`')}\`)">
                <div class="history-query">${escapeHtml(item.query)}</div>
                <div class="history-meta">
                    <span class="badge ${item.success ? 'badge-success' : 'badge-info'}">${item.success ? 'Success' : 'Failed'}</span>
                    <span>⚡ ${item.timeMs} ms</span>
                    <span>${item.timestamp}</span>
                </div>
            </div>
        `).join('');
    }

    window.loadHistoryQuery = function(queryStr) {
        appState.editor.setValue(queryStr);
        document.querySelector('.tab-btn[data-target="queryResults"]').click();
    };

    function escapeHtml(text) {
        if (text === null || text === undefined) return '';
        return String(text)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    }
});
