let downloadsActive = false;
let numDownloads = 0;
let numDownloadsCompleted = 0;
let allSize = '';

function scheduleContentUpdate() {
    setTimeout(() => {
        if (!downloadsActive) {
            updateContent();
        }
        scheduleContentUpdate();
    }, 2000);
}

updateContent();
scheduleContentUpdate();

function downloadButtonString() {
    if (downloadsActive) {
        return 'Downloading...(' + numDownloadsCompleted + ' / ' + numDownloads + ')'; 
    } else {
        return 'Download All Files ' + allSize;
    }
}

document.getElementById('downloadAll').addEventListener('click', async () => {
    const links = document.querySelectorAll('a.link');
    const button = document.getElementById('downloadAll');
    const originalText = button.textContent;

    numDownloadsCompleted = 0;
    numDownloads = links.length;
    button.disabled = true;
    downloadsActive = true;
    button.textContent = downloadButtonString();
    
    await Promise.all(Array.from(links).map(link => downloadFile(link)));

    button.textContent = originalText;
    downloadsActive = false;
    button.disabled = false;
});

async function downloadFile(link) {
    await fetch('/download-all');
    const url = link.href.slice(0, -1) + '0';

    let isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);
    let isLocal = /(^127\.)|(^192\.168\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^::1$)|(^[fF][cCdD])/i.test(location.hostname);

    if (isMobile && !isLocal) { 
        try {
            window.open(url, '_blank');
        } catch (error) {
            console.error(error);
        }
    } else {
        try {
            const response = await fetch(url);
            const blob = await response.blob();
            const contentDisposition = response.headers.get('content-disposition');
            const fileName = getFileNameFromContentDisposition(contentDisposition);
            const a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = fileName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
        } catch (error) {
            console.error(error);
        }
    }

    numDownloadsCompleted++;
    document.getElementById('downloadAll').textContent = downloadButtonString();
}

function getFileNameFromContentDisposition(contentDisposition) {
    const fileNameMatch = contentDisposition && contentDisposition.match(/filename="(.+)"/);
    return fileNameMatch ? fileNameMatch[1] : 'unknown';
}

function to_color(r, g, b) {
    return 'rgb(' + r + ', ' + g + ', ' + b + ')';
}

async function updateContent() {
    let button = document.getElementById('downloadAll');
    try {
        const response = await fetch('/update-content');
        const json = await response.json();
        let html = json.html;
        let size = json.size;
        let primary = json.primary;
        let darker_primary = {r: Math.floor(primary.r * 0.8), g: Math.floor(primary.g * 0.8), b: Math.floor(primary.b * 0.8)};
        let primary_color = to_color(primary.r, primary.g, primary.b);
        let darker_primary_color = to_color(darker_primary.r, darker_primary.g, darker_primary.b);
        let background = json.background;
        let background_darker = {r: Math.floor(background.r * 0.8), g: Math.floor(background.g * 0.8), b: Math.floor(background.b * 0.8)};
        let background_color = to_color(background.r, background.g, background.b);
        let background_darker_color = to_color(background_darker.r, background_darker.g, background_darker.b);
        let text = json.text;
        let text_inverse = {r: 255, g: 255, b: 255};
        if ((primary.r + primary.g + primary.b) > 382) {
            text_inverse = {r: 0, g: 0, b: 0};
        } 
        let text_color = to_color(text.r, text.g, text.b);
        let text_inverse_color = to_color(text_inverse.r, text_inverse.g, text_inverse.b);

        document.documentElement.style.setProperty('--primary-color', primary_color);
        document.documentElement.style.setProperty('--secondary-color', darker_primary_color);
        document.documentElement.style.setProperty('--background-color', background_color);
        document.documentElement.style.setProperty('--background-color-2', background_darker_color);
        document.documentElement.style.setProperty('--text-color', text_color);
        document.documentElement.style.setProperty('--text-color-2', text_inverse_color);
        allSize = size;
        document.getElementById('fileList').innerHTML = html;
        document.getElementById('downloadAll').textContent = "Download All Files (" + size + ")";

        button.hidden = false;
    } catch (error) {
        button.hidden = true;
        document.getElementById('fileList').innerHTML = "<h2>No Files available</h2>";
    }
}

