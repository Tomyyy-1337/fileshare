document.getElementById('downloadAll').addEventListener('click', function() {
    const links = document.querySelectorAll('a.link');
    links.forEach(link => {
        const url = link.href;
        const fileName = link.previousElementSibling.textContent;
        fetch(url)
            .then(response => response.blob())
            .then(blob => {
                const a = document.createElement('a');
                a.href = URL.createObjectURL(blob);
                a.download = fileName;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
            })
            .catch(console.error);
    });
});