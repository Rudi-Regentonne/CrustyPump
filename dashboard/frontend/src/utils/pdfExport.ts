import { jsPDF } from 'jspdf';
import type { Chart } from 'chart.js';

export interface PdfChartEntry {
  title: string;
  chart: Chart | null;
  canvas: HTMLCanvasElement | null;
  container: HTMLDivElement | null;
}

function captureChartForPage(
  entry: PdfChartEntry,
  pageRatio: number,
): string | null {
  const { chart, canvas, container } = entry;
  if (!canvas || canvas.width === 0 || canvas.height === 0) {
    return null;
  }

  if (!chart || !container || container.clientWidth === 0) {
    return canvas.toDataURL('image/jpeg', 0.92);
  }

  const prevWidth = container.style.width;
  const prevHeight = container.style.height;
  const prevAnimation = chart.options.animation;

  const targetW = 1400;
  const targetH = Math.round(targetW / pageRatio);

  chart.options.animation = false;
  container.style.width = `${targetW}px`;
  container.style.height = `${targetH}px`;
  chart.resize(targetW, targetH);
  chart.update('none');

  // Composite onto a solid white background so transparent areas render cleanly in PDF.
  const exportCanvas = document.createElement('canvas');
  exportCanvas.width = canvas.width;
  exportCanvas.height = canvas.height;
  const exportCtx = exportCanvas.getContext('2d');
  if (exportCtx) {
    exportCtx.fillStyle = '#ffffff';
    exportCtx.fillRect(0, 0, exportCanvas.width, exportCanvas.height);
    exportCtx.drawImage(canvas, 0, 0);
  }
  const imgData = exportCanvas.toDataURL('image/jpeg', 0.92);

  container.style.width = prevWidth;
  container.style.height = prevHeight;
  chart.options.animation = prevAnimation;
  chart.resize();
  chart.update('none');

  return imgData;
}

export function exportChartsToPdf(entries: PdfChartEntry[]): boolean {
  if (entries.length === 0) {
    return false;
  }

  const pdf = new jsPDF({ orientation: 'landscape', unit: 'mm', format: 'a4' });
  const pageWidth = pdf.internal.pageSize.getWidth();
  const pageHeight = pdf.internal.pageSize.getHeight();
  const margin = 12;

  entries.forEach((entry, index) => {
    if (index > 0) pdf.addPage();

    pdf.setFont('helvetica', 'bold');
    pdf.setFontSize(16);
    pdf.setTextColor(40, 40, 40);
    pdf.text(entry.title || 'Chart', pageWidth / 2, margin + 4, {
      align: 'center',
      maxWidth: pageWidth - margin * 2,
    });

    const maxW = pageWidth - margin * 2;
    const maxH = pageHeight - margin * 2 - 14;
    const imgData = captureChartForPage(entry, maxW / maxH);
    if (imgData) {
      pdf.addImage(imgData, 'JPEG', margin, margin + 10, maxW, maxH, undefined, 'FAST');
    }
  });

  pdf.save('heatpump_report.pdf');
  return true;
}
