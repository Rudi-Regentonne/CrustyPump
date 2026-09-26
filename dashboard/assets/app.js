const API_BASE = '/api';
let tiles = [];
let sortable = null;
let charts = {};

// DOM Elements
const timeRangePreset = document.getElementById('time-range-preset');
const customTimeContainer = document.getElementById('custom-time-container');
const dtStart = document.getElementById('datetime-start');
const dtEnd = document.getElementById('datetime-end');
const granularity = document.getElementById('granularity');
const channelSelector = document.getElementById('channel-selector');
const addBtn = document.getElementById('add-btn');
const refreshBtn = document.getElementById('refresh-btn');
const exportBtn = document.getElementById('export-btn');
const saveBtn = document.getElementById('save-btn');
const loadBtn = document.getElementById('load-btn');
const statusMsg = document.getElementById('status-msg');
const tilesContainer = document.getElementById('tiles-container');
const emptyState = document.getElementById('empty-state');

// Initialize
async function init() {
    initSortable();
    await loadChannels();
    await loadConfig();
    setupEventListeners();
}

function initSortable() {
    sortable = new Sortable(tilesContainer, {
        handle: '.tile-header',
        animation: 150,
        onEnd: function (evt) {
            const item = tiles.splice(evt.oldIndex, 1)[0];
            tiles.splice(evt.newIndex, 0, item);
        }
    });
}

// Data fetching
async function loadChannels() {
    try {
        const res = await fetch(`${API_BASE}/channels`);
        const channels = await res.json();
        
        channelSelector.innerHTML = '<option value="">Select a channel...</option>';
        channels.forEach(c => {
            const opt = document.createElement('option');
            opt.value = `${c.measurement}:${c.channel}`;
            opt.textContent = `${c.measurement} / ${c.channel}`;
            channelSelector.appendChild(opt);
        });
    } catch (e) {
        console.error("Failed to load channels", e);
    }
}

async function loadConfig() {
    try {
        const res = await fetch(`${API_BASE}/config`);
        const config = await res.json();
        
        if (config.time_range_preset) {
            timeRangePreset.value = config.time_range_preset;
        }
        if (config.time_range_preset === 'custom' && config.time_range) {
            dtStart.value = config.time_range.start;
            dtEnd.value = config.time_range.end;
            customTimeContainer.classList.remove('hidden');
        }
        if (config.granularity) {
            granularity.value = config.granularity;
        }
        
        if (config.tiles && config.tiles.length > 0) {
            tiles = config.tiles;
            renderTiles();
        }
    } catch (e) {
        console.error("Failed to load config", e);
    }
}

async function saveConfig() {
    const config = {
        time_range_preset: timeRangePreset.value,
        time_range: {
            start: dtStart.value,
            end: dtEnd.value
        },
        granularity: granularity.value,
        tiles: tiles
    };
    
    try {
        const res = await fetch(`${API_BASE}/config`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(config)
        });
        if (res.ok) {
            showStatus("Layout saved");
        } else {
            showStatus("Failed to save", true);
        }
    } catch (e) {
        showStatus("Error saving", true);
    }
}

function showStatus(msg, isError = false) {
    statusMsg.textContent = msg;
    statusMsg.style.color = isError ? '#d32f2f' : '#4caf50';
    setTimeout(() => statusMsg.textContent = '', 3000);
}

// UI Logic
function setupEventListeners() {
    timeRangePreset.addEventListener('change', (e) => {
        if (e.target.value === 'custom') {
            customTimeContainer.classList.remove('hidden');
            if (!dtStart.value) {
                const now = new Date();
                const past = new Date(now.getTime() - 60 * 60 * 1000);
                dtEnd.value = toLocalISOString(now);
                dtStart.value = toLocalISOString(past);
            }
        } else {
            customTimeContainer.classList.add('hidden');
        }
    });

    addBtn.addEventListener('click', () => {
        const val = channelSelector.value;
        if (!val) return;
        
        const [measurement, channel] = val.split(':');
        const exists = tiles.some(t => t.channel === channel && t.measurement === measurement);
        
        if (!exists) {
            tiles.push({ channel, measurement });
            renderTiles();
        }
    });

    refreshBtn.addEventListener('click', () => {
        refreshAllCharts();
    });

    saveBtn.addEventListener('click', saveConfig);
    loadBtn.addEventListener('click', async () => {
        tiles = [];
        renderTiles();
        await loadConfig();
    });

    exportBtn.addEventListener('click', () => {
        const activeTiles = document.querySelectorAll('.tile');
        if (activeTiles.length === 0) return;
        if (!window.jspdf) {
            showStatus('PDF library not loaded', true);
            return;
        }

        const { jsPDF } = window.jspdf;
        const pdf = new jsPDF({ orientation: 'landscape', unit: 'mm', format: 'a4' });
        const pageWidth = pdf.internal.pageSize.getWidth();
        const pageHeight = pdf.internal.pageSize.getHeight();
        const margin = 12;

        activeTiles.forEach((tile, index) => {
            if (index > 0) pdf.addPage();

            const titleText = tile.querySelector('.tile-title')?.innerText || 'Chart';
            pdf.setFont('helvetica', 'bold');
            pdf.setFontSize(16);
            pdf.setTextColor(40, 40, 40);
            pdf.text(titleText, pageWidth / 2, margin + 4, {
                align: 'center',
                maxWidth: pageWidth - margin * 2,
            });

            const canvas = tile.querySelector('canvas');
            if (!canvas || canvas.width === 0 || canvas.height === 0) return;

            const imgData = canvas.toDataURL('image/png');
            const maxW = pageWidth - margin * 2;
            const maxH = pageHeight - margin * 2 - 14;
            const ratio = canvas.width / canvas.height;
            let w = maxW;
            let h = w / ratio;
            if (h > maxH) {
                h = maxH;
                w = h * ratio;
            }
            pdf.addImage(imgData, 'PNG', (pageWidth - w) / 2, margin + 10, w, h);
        });

        pdf.save('heatpump_report.pdf');
    });
}

function localInputToUtcIso(value) {
    if (!value) return value;
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    return parsed.toISOString();
}

function toLocalISOString(date) {
    const tzOffset = (new Date()).getTimezoneOffset() * 60000;
    return (new Date(date - tzOffset)).toISOString().slice(0, -5);
}

function getStartEndDates() {
    const preset = timeRangePreset.value;
    if (preset === 'custom') {
        return {
            start: localInputToUtcIso(dtStart.value),
            end: localInputToUtcIso(dtEnd.value),
        };
    }
    
    const end = new Date();
    const start = new Date(end);
    
    if (preset === '1m') start.setMinutes(start.getMinutes() - 1);
    else if (preset === '2m') start.setMinutes(start.getMinutes() - 2);
    else if (preset === '15m') start.setMinutes(start.getMinutes() - 15);
    else if (preset === '1h') start.setHours(start.getHours() - 1);
    else if (preset === '24h') start.setHours(start.getHours() - 24);
    else if (preset === '7d') start.setDate(start.getDate() - 7);
    
    return {
        start: start.toISOString(),
        end: end.toISOString()
    };
}

// Chart Logic
function renderTiles() {
    // Clear existing
    Object.values(charts).forEach(c => c.destroy());
    charts = {};
    tilesContainer.innerHTML = '';
    
    if (tiles.length === 0) {
        tilesContainer.appendChild(emptyState);
        emptyState.style.display = 'block';
        return;
    }
    
    emptyState.style.display = 'none';
    
    tiles.forEach((t, i) => {
        const tileId = `tile-${t.measurement}-${t.channel.replace(/\W+/g, '-')}`;

        const tileDiv = document.createElement('div');
        tileDiv.className = 'tile';

        const header = document.createElement('div');
        header.className = 'tile-header';
        const title = document.createElement('div');
        title.className = 'tile-title';
        title.textContent = `${t.channel} (${t.measurement})`;
        const controls = document.createElement('div');
        controls.className = 'tile-controls';
        const removeBtn = document.createElement('button');
        removeBtn.className = 'remove';
        removeBtn.type = 'button';
        removeBtn.textContent = '×';
        removeBtn.addEventListener('click', () => {
            tiles.splice(i, 1);
            renderTiles();
        });
        controls.appendChild(removeBtn);
        header.append(title, controls);

        const chartBox = document.createElement('div');
        chartBox.className = 'chart-container';
        const canvas = document.createElement('canvas');
        canvas.id = tileId;
        chartBox.appendChild(canvas);
        tileDiv.append(header, chartBox);
        tilesContainer.appendChild(tileDiv);
        
        // Create chart
        const ctx = document.getElementById(tileId).getContext('2d');
        charts[tileId] = new Chart(ctx, {
            type: 'line',
            data: { datasets: [] },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                interaction: {
                    mode: 'index',
                    intersect: false,
                },
                scales: {
                    x: {
                        type: 'time',
                        time: { tooltipFormat: 'PPpp' },
                        title: { display: false }
                    },
                    y: {
                        title: { display: true, text: 'Value' }
                    }
                },
                plugins: {
                    legend: { display: false }
                }
            }
        });
        
        // Load data
        loadChartData(t, tileId);
    });
}

function refreshAllCharts() {
    tiles.forEach(t => {
        const tileId = `tile-${t.measurement}-${t.channel.replace(/\\W+/g, '-')}`;
        if (charts[tileId]) {
            loadChartData(t, tileId);
        }
    });
}

async function loadChartData(tile, tileId) {
    const dates = getStartEndDates();
    if (!dates.start || !dates.end) return;
    
    const gran = granularity.value;
    const url = `${API_BASE}/data?channel=${encodeURIComponent(tile.channel)}&measurement=${encodeURIComponent(tile.measurement)}&start=${encodeURIComponent(dates.start)}&end=${encodeURIComponent(dates.end)}&granularity=${encodeURIComponent(gran)}`;
    
    try {
        const res = await fetch(url);
        const data = await res.json();
        
        const chartData = data.map(d => ({
            x: new Date(d.bucket),
            y: d.val
        }));
        
        const chart = charts[tileId];
        if (chart) {
            chart.data.datasets = [{
                label: tile.channel,
                data: chartData,
                borderColor: '#1976D2',
                backgroundColor: 'rgba(25, 118, 210, 0.1)',
                borderWidth: 2,
                pointRadius: 0,
                pointHitRadius: 10,
                fill: true,
                tension: 0.1
            }];
            chart.update();
        }
    } catch (e) {
        console.error("Failed to load chart data", e);
    }
}

// Start
document.addEventListener('DOMContentLoaded', init);
