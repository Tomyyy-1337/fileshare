let downloadsActive = false;
let numDownloads = 0;
let numDownloadsCompleted = 0;

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
        return 'Download All';
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
    try {
        const response = await fetch('/update-content');
        const json = await response.json();
        let html = json.html;
        let size = json.size;
        document.getElementById('fileList').innerHTML = html;
        document.getElementById('downloadAll').textContent = "Download All Files (" + size + ")";
    } catch (error) {
        document.getElementById('fileList').innerHTML = "<h2>No Files available</h2>";
    }
}