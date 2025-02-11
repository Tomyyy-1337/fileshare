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
    numDownloadsCompleted = 0;
    numDownloads = links.length;
    const button = document.getElementById('downloadAll');
    const originalText = button.textContent;
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
    const fileName = link.previousElementSibling.textContent;

    try {
        const response = await fetch(url);
        const blob = await response.blob();
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

async function updateContent() {
    try {
        const response = await fetch('/update-content');
        const html = await response.text();
        document.getElementById('fileList').innerHTML = html;
    } catch (error) {
        document.getElementById('fileList').innerHTML = "<h2>No Files available</h2>";
    }
}