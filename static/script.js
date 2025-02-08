let downloadsActive = false;

function scheduleContentUpdate() {
    setTimeout(() => {
        if (!downloadsActive) {
            updateContent();
        } 
        scheduleContentUpdate();
    }, 2000);
}

scheduleContentUpdate();

document.getElementById('downloadAll').addEventListener('click', async () => {
    const button = document.getElementById('downloadAll');
    const originalText = button.textContent;
    button.textContent = 'Downloading...';
    button.disabled = true;

    const links = document.querySelectorAll('a.link');
    downloadsActive = true;

    await Promise.all(Array.from(links).map(link => downloadFile(link)));

    downloadsActive = false;
    button.textContent = originalText;
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