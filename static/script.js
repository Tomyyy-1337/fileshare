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
    await fetch('/download-all');
    const links = Array.from(document.querySelectorAll('a.link'));
    const button = document.getElementById('downloadAll');
    const originalText = button.textContent;
    
    numDownloadsCompleted = 0;
    numDownloads = links.length;
    button.disabled = true;
    downloadsActive = true;
    button.textContent = downloadButtonString();
    
    const queue = [...links];
    const activeDownloads = new Set();
    
    async function processQueue() {
        while (queue.length > 0) {
            if (activeDownloads.size < 5) {
                const link = queue.shift();
                const downloadPromise = downloadFile(link).then(() => {
                    activeDownloads.delete(downloadPromise);
                    processQueue(); 
                });
                activeDownloads.add(downloadPromise);
            }
            await Promise.race(activeDownloads);
        }
    }
    
    await processQueue();
    await Promise.all(activeDownloads);
    
    button.textContent = originalText;
    downloadsActive = false;
    button.disabled = false;
});


async function downloadFile(link) {
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

function to_color(color) {
    return 'rgb(' + color.r + ', ' + color.g + ', ' + color.b + ')';
}

async function updateContent() {
    let button = document.getElementById('downloadAll');
    try {
        const response = await fetch('/update-content');
        const json = await response.json();
        let html = json.html;
        let size = json.size;

        document.documentElement.style.setProperty('--primary-color', to_color(json.primary));
        document.documentElement.style.setProperty('--secondary-color', to_color(json.secondary));
        document.documentElement.style.setProperty('--background-color', to_color(json.background));
        document.documentElement.style.setProperty('--background-color-2', to_color(json.dark_background));
        document.documentElement.style.setProperty('--text-color', to_color(json.text));
        document.documentElement.style.setProperty('--text-color-2', to_color(json.text_secondary));
        allSize = size;
        document.getElementById('fileList').innerHTML = html;
        document.getElementById('downloadAll').textContent = "Download All Files (" + size + ")";

        button.hidden = false;
    } catch (error) {
        button.hidden = true;
        document.getElementById('fileList').innerHTML = "<h2>No Files available</h2>";
    }
}

