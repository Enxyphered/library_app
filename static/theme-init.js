// Immediately apply persisted or system theme from cookie (fallback to localStorage or system preference)
(() => {
    function getThemeCookie() {
        const match = document.cookie.match(/(?:^|;\s*)theme=([^;]+)/);
        return match ? decodeURIComponent(match[1]) : null;
    }

    try {
        const cookieTheme = getThemeCookie();
        const storedTheme = cookieTheme || localStorage.getItem('theme');
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        
        if (storedTheme === 'dark' || (!storedTheme && prefersDark)) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    } catch (_) {}
})();
