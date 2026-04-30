(() => {
  'use strict';

  // ===== Mobile Nav Toggle =====
  const hamburger = document.getElementById('nav-hamburger');
  const navLinks = document.getElementById('nav-links');

  if (hamburger && navLinks) {
    hamburger.addEventListener('click', () => {
      hamburger.classList.toggle('active');
      navLinks.classList.toggle('active');
    });
    navLinks.querySelectorAll('a').forEach(link => {
      link.addEventListener('click', () => {
        hamburger.classList.remove('active');
        navLinks.classList.remove('active');
      });
    });
  }

  // ===== Scroll Fade-In =====
  const fadeEls = document.querySelectorAll('.fade-in');
  if (fadeEls.length) {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          observer.unobserve(entry.target);
        }
      });
    }, { threshold: 0.12 });
    fadeEls.forEach(el => observer.observe(el));
  }

  // ===== FAQ Accordion =====
  document.querySelectorAll('.faq-question').forEach(btn => {
    btn.addEventListener('click', () => {
      const item = btn.closest('.faq-item');
      const answer = item.querySelector('.faq-answer');
      const isActive = item.classList.contains('active');

      document.querySelectorAll('.faq-item').forEach(fi => {
        fi.classList.remove('active');
        fi.querySelector('.faq-answer').style.maxHeight = null;
      });

      if (!isActive) {
        item.classList.add('active');
        answer.style.maxHeight = answer.scrollHeight + 'px';
      }
    });
  });

  // ===== Platform Detection =====
  const detectPlatform = () => {
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes('mac')) return 'macos';
    if (ua.includes('win')) return 'windows';
    if (ua.includes('linux')) return 'linux';
    return null;
  };

  const platform = detectPlatform();
  if (platform) {
    const card = document.querySelector(`[data-platform="${platform}"]`);
    if (card) card.classList.add('recommended');
  }

  // ===== Fetch Latest Release & Set Direct Download URLs =====
  const REPO = 'andiharka/siputar';
  const FALLBACK = `https://github.com/${REPO}/releases/latest`;

  // Map of button IDs to the file extension they should download
  const BUTTON_MAP = {
    'dl-macos':   { match: (name) => name.startsWith('darwin_') && name.endsWith('.dmg') },
    'dl-windows': { match: (name) => name.startsWith('windows_') && name.endsWith('.msi') },
    'dl-linux-deb': { match: (name) => name.startsWith('linux_') && name.endsWith('.deb') },
    'dl-linux-rpm': { match: (name) => name.startsWith('linux_') && name.endsWith('.rpm') },
  };

  fetch(`https://api.github.com/repos/${REPO}/releases/latest`)
    .then(r => r.json())
    .then(data => {
      if (!data.assets) return;

      // Update version badges
      if (data.tag_name) {
        document.querySelectorAll('.version-badge').forEach(badge => {
          badge.textContent = data.tag_name;
        });
      }

      // Set direct download URLs on buttons
      for (const [btnId, config] of Object.entries(BUTTON_MAP)) {
        const asset = data.assets.find(a => config.match(a.name));
        const btn = document.getElementById(btnId);
        if (asset && btn) {
          btn.href = asset.browser_download_url;
        }
      }
    })
    .catch(() => {
      // On error, buttons keep their fallback URLs pointing to /releases/latest
    });
})();
