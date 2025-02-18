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

document.addEventListener('DOMContentLoaded', function() {
    const downloadLinks = document.querySelectorAll('.download_single');

    downloadLinks.forEach(link => {
        link.addEventListener('click', async function(event) {
            event.preventDefault();
            const url = this.href;
            const fileName = this.previousElementSibling.textContent.trim();

            try {
                const response = await fetch(url);
                if (!response.ok) {
                    throw new Error('Network response was not ok');
                }
                const blob = await response.blob();
                const link = document.createElement('a');
                link.href = window.URL.createObjectURL(blob);
                link.download = fileName;
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
            } catch (error) {
                console.error('There has been a problem with your fetch operation:', error);
            }
        });
    });
});

async function downloadFile(link) {
    const url = link.href;

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
    numDownloadsCompleted++;
    document.getElementById('downloadAll').textContent = downloadButtonString();
}

function getFileNameFromContentDisposition(contentDisposition) {
    const fileNameMatch = contentDisposition && contentDisposition.match(/filename="(.+)"/);
    return fileNameMatch ? fileNameMatch[1] : 'unknown';
}

async function updateContent() {
    let button = document.getElementById('downloadAll');
    try {
        const response = await fetch('/update-content');
        const json = await response.json();
        let html = json.html;
        let size = json.size;
        allSize = size;
        document.getElementById('fileList').innerHTML = html;
        document.getElementById('downloadAll').textContent = "Download All Files (" + size + ")";
        button.hidden = false;
    } catch (error) {
        button.hidden = true;
        document.getElementById('fileList').innerHTML = "<h2>No Files available</h2>";
    }
}

