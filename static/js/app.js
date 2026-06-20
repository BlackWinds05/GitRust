// GitRust client-side interactivity
document.addEventListener('DOMContentLoaded', () => {
    // Flash message auto-dismiss
    document.querySelectorAll('.flash-message').forEach(el => {
        setTimeout(() => { el.style.opacity = '0'; el.style.transition = 'opacity 0.3s'; }, 5000);
    });

    // Captcha refresh button
    const captchaRefresh = document.getElementById('captcha-refresh');
    if (captchaRefresh) {
        captchaRefresh.addEventListener('click', async () => {
            const img = document.getElementById('captcha-img');
            const token = document.getElementById('captcha-token');
            try {
                const resp = await fetch('/auth/captcha/refresh', { method: 'POST' });
                const data = await resp.json();
                img.src = data.image;
                token.value = data.token;
            } catch (e) { console.error('captcha refresh failed', e); }
        });
    }

    // Generic data-confirm handler for destructive actions
    document.querySelectorAll('[data-confirm]').forEach(function(el) {
        el.addEventListener('submit', function(e) {
            if (!confirm(el.dataset.confirm)) {
                e.preventDefault();
            }
        });
    });

    // Inline edit toggle (for labels, milestones, etc.)
    document.querySelectorAll('[data-toggle-edit]').forEach(function(btn) {
        btn.addEventListener('click', function() {
            const target = document.getElementById(btn.dataset.toggleEdit);
            if (target) {
                target.style.display = target.style.display === 'none' ? 'block' : 'none';
            }
        });
    });
});
