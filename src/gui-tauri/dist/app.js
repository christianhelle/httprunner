const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

// Application state
let currentFile = null;
let currentDir = null;
let fileContent = '';
let isModified = false;

// Initialize the application
async function init() {
    try {
        currentDir = await invoke('get_root_directory');
        updateFooter();
        await refreshFileList();
    } catch (error) {
        console.error('Failed to initialize:', error);
    }

    setupEventListeners();
}

function setupEventListeners() {
    // Open folder button
    document.getElementById('open-folder-btn').addEventListener('click', async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
            });
            
            if (selected) {
                await invoke('set_root_directory', { path: selected });
                currentDir = selected;
                currentFile = null;
                updateFooter();
                await refreshFileList();
                clearEditor();
                clearResults();
            }
        } catch (error) {
            console.error('Failed to open folder:', error);
        }
    });

    // Environment selector
    document.getElementById('env-select').addEventListener('change', async (e) => {
        const env = e.target.value || null;
        try {
            await invoke('set_environment', { env });
        } catch (error) {
            console.error('Failed to set environment:', error);
        }
    });

    // Save button
    document.getElementById('save-btn').addEventListener('click', async () => {
        if (currentFile && isModified) {
            try {
                const content = document.getElementById('file-editor').value;
                await invoke('write_file_content', { path: currentFile, content });
                fileContent = content;
                isModified = false;
                updateSaveButton();
                showStatus('File saved successfully', 'success');
                await loadFileRequests(currentFile);
            } catch (error) {
                console.error('Failed to save file:', error);
                showStatus('Failed to save file', 'error');
            }
        }
    });

    // File editor change detection
    document.getElementById('file-editor').addEventListener('input', (e) => {
        isModified = e.target.value !== fileContent;
        updateSaveButton();
    });

    // Run all requests button
    document.getElementById('run-all-btn').addEventListener('click', async () => {
        if (currentFile && !isModified) {
            await runAllRequests();
        }
    });
}

async function refreshFileList() {
    try {
        const files = await invoke('list_http_files');
        const fileListEl = document.getElementById('file-list');
        
        if (files.length === 0) {
            fileListEl.innerHTML = '<p class="empty-state">No .http files found</p>';
            return;
        }
        
        fileListEl.innerHTML = '';
        files.forEach(file => {
            const item = document.createElement('div');
            item.className = 'file-item';
            item.textContent = file.name;
            item.dataset.path = file.path;
            
            if (currentFile === file.path) {
                item.classList.add('selected');
            }
            
            item.addEventListener('click', () => selectFile(file.path));
            fileListEl.appendChild(item);
        });
    } catch (error) {
        console.error('Failed to list files:', error);
    }
}

async function selectFile(path) {
    currentFile = path;
    
    // Update selected state in file list
    document.querySelectorAll('.file-item').forEach(item => {
        item.classList.toggle('selected', item.dataset.path === path);
    });
    
    try {
        // Load file content
        const content = await invoke('read_file_content', { path });
        fileContent = content;
        isModified = false;
        
        document.getElementById('file-editor').value = content;
        document.getElementById('file-path').textContent = path;
        updateSaveButton();
        updateRunAllButton();
        
        // Load requests list
        await loadFileRequests(path);
        
        // Load environments
        await loadEnvironments(path);
        
        // Update footer
        updateFooter();
        
        // Save selected file
        await invoke('select_file', { path });
    } catch (error) {
        console.error('Failed to load file:', error);
        showStatus('Failed to load file', 'error');
    }
}

async function loadFileRequests(path) {
    try {
        const requests = await invoke('parse_http_file', { path });
        const requestsListEl = document.getElementById('requests-list');
        
        if (requests.length === 0) {
            requestsListEl.innerHTML = '<p class="empty-state">No requests found</p>';
            return;
        }
        
        requestsListEl.innerHTML = '';
        requests.forEach(req => {
            const item = document.createElement('div');
            item.className = 'request-item';
            
            const header = document.createElement('div');
            if (req.name) {
                const nameEl = document.createElement('div');
                nameEl.style.fontWeight = '600';
                nameEl.style.marginBottom = '4px';
                nameEl.textContent = req.name;
                header.appendChild(nameEl);
            }
            
            const methodUrl = document.createElement('div');
            methodUrl.innerHTML = `<span class="method">${req.method}</span>`;
            
            const urlEl = document.createElement('div');
            urlEl.className = 'url';
            urlEl.textContent = req.url;
            
            item.appendChild(header);
            item.appendChild(methodUrl);
            item.appendChild(urlEl);
            
            const runBtn = document.createElement('button');
            runBtn.className = 'btn btn-success btn-small run-btn';
            runBtn.textContent = '▶ Run';
            runBtn.addEventListener('click', () => runSingleRequest(req.index));
            item.appendChild(runBtn);
            
            requestsListEl.appendChild(item);
        });
    } catch (error) {
        console.error('Failed to parse file:', error);
    }
}

async function loadEnvironments(path) {
    try {
        const envs = await invoke('list_environments', { path });
        const envSelect = document.getElementById('env-select');
        const currentEnv = await invoke('get_environment');
        
        envSelect.innerHTML = '<option value="">None</option>';
        envs.forEach(env => {
            const option = document.createElement('option');
            option.value = env;
            option.textContent = env;
            if (env === currentEnv) {
                option.selected = true;
            }
            envSelect.appendChild(option);
        });
    } catch (error) {
        console.error('Failed to load environments:', error);
    }
}

async function runSingleRequest(index) {
    if (!currentFile) return;
    
    try {
        const env = document.getElementById('env-select').value || null;
        showStatus('Running request...', 'info');
        
        const result = await invoke('run_single_request', {
            path: currentFile,
            index,
            environment: env
        });
        
        displayResults([result]);
        showStatus('Request completed', 'success');
    } catch (error) {
        console.error('Failed to run request:', error);
        showStatus('Failed to run request', 'error');
    }
}

async function runAllRequests() {
    if (!currentFile) return;
    
    try {
        const env = document.getElementById('env-select').value || null;
        showStatus('Running all requests...', 'info');
        
        const results = await invoke('run_all_requests', {
            path: currentFile,
            environment: env
        });
        
        displayResults(results);
        showStatus(`Completed ${results.length} requests`, 'success');
    } catch (error) {
        console.error('Failed to run all requests:', error);
        showStatus('Failed to run requests', 'error');
    }
}

function displayResults(results) {
    const resultsAreaEl = document.getElementById('results-area');
    resultsAreaEl.innerHTML = '';
    
    if (results.length === 0) {
        resultsAreaEl.innerHTML = '<p class="empty-state">No results</p>';
        return;
    }
    
    results.forEach(result => {
        const item = document.createElement('div');
        item.className = `result-item ${result.success ? 'success' : 'failure'}`;
        
        const header = document.createElement('div');
        header.className = 'result-header';
        
        const methodEl = document.createElement('span');
        methodEl.className = 'method';
        methodEl.textContent = result.method;
        
        const statusEl = document.createElement('span');
        statusEl.className = 'status';
        if (result.success && result.status) {
            statusEl.textContent = `${result.status} (${result.duration_ms}ms)`;
        } else {
            statusEl.textContent = 'Failed';
        }
        
        header.appendChild(methodEl);
        header.appendChild(statusEl);
        
        const urlEl = document.createElement('div');
        urlEl.className = 'url';
        urlEl.textContent = result.url;
        
        item.appendChild(header);
        item.appendChild(urlEl);
        
        if (result.error) {
            const errorEl = document.createElement('div');
            errorEl.className = 'error';
            errorEl.textContent = `Error: ${result.error}`;
            item.appendChild(errorEl);
        } else if (result.response_body) {
            const bodyEl = document.createElement('div');
            bodyEl.className = 'response-body';
            
            try {
                // Try to format JSON
                const json = JSON.parse(result.response_body);
                bodyEl.textContent = JSON.stringify(json, null, 2);
            } catch {
                // Not JSON, display as-is
                bodyEl.textContent = result.response_body;
            }
            
            item.appendChild(bodyEl);
        }
        
        resultsAreaEl.appendChild(item);
    });
}

function clearEditor() {
    document.getElementById('file-editor').value = '';
    document.getElementById('file-path').textContent = '';
    document.getElementById('requests-list').innerHTML = '<p class="empty-state">Select a file to view requests</p>';
    fileContent = '';
    isModified = false;
    updateSaveButton();
    updateRunAllButton();
}

function clearResults() {
    document.getElementById('results-area').innerHTML = '<p class="empty-state">No results yet</p>';
}

function updateFooter() {
    document.getElementById('current-dir').textContent = currentDir || '';
    document.getElementById('selected-file-footer').textContent = currentFile ? `Selected: ${currentFile}` : '';
}

function updateSaveButton() {
    const saveBtn = document.getElementById('save-btn');
    saveBtn.disabled = !isModified;
}

function updateRunAllButton() {
    const runAllBtn = document.getElementById('run-all-btn');
    runAllBtn.disabled = !currentFile || isModified;
}

function showStatus(message, type) {
    const statusEl = document.getElementById('status-message');
    statusEl.textContent = message;
    statusEl.style.color = type === 'error' ? '#dc3545' : type === 'success' ? '#28a745' : '#666';
    
    setTimeout(() => {
        statusEl.textContent = '';
    }, 3000);
}

// Initialize when the page loads
window.addEventListener('DOMContentLoaded', init);
