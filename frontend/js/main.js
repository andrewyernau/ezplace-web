document.addEventListener('DOMContentLoaded', function() {
    // Modal close functionality
    document.addEventListener('click', function(event) {
        if (event.target.classList.contains('close') || event.target == document.getElementById('login-modal')) {
            document.getElementById('login-modal').style.display = 'none';
        }
    });

    // Mobile Menu Toggle
    const hamburger = document.querySelector('.hamburger');
    const navLinks = document.querySelector('.nav-links');

    hamburger.addEventListener('click', function() {
        navLinks.classList.toggle('active');
        hamburger.classList.toggle('active');
    });

    // Copy server IP
    const copyIpBtn = document.getElementById('copy-ip');
    
    copyIpBtn.addEventListener('click', function() {
        const ip = this.getAttribute('data-ip');
        navigator.clipboard.writeText(ip).then(function() {
            const originalText = copyIpBtn.textContent;
            copyIpBtn.textContent = 'Copied!';
            setTimeout(function() {
                copyIpBtn.textContent = originalText;
            }, 2000);
        });
    });

    // Scroll to Top Button
    const scrollTopBtn = document.getElementById('scroll-top');
    
    window.addEventListener('scroll', function() {
        if (window.pageYOffset > 300) {
            scrollTopBtn.style.display = 'flex';
        } else {
            scrollTopBtn.style.display = 'none';
        }
    });

    scrollTopBtn.addEventListener('click', function() {
        window.scrollTo({
            top: 0,
            behavior: 'smooth'
        });
    });

    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            if (this.getAttribute('href') !== '#') {
                e.preventDefault();
                const target = document.querySelector(this.getAttribute('href'));
                if (target) {
                    window.scrollTo({
                        top: target.offsetTop - 80,
                        behavior: 'smooth'
                    });
                }
            }
        });
    });

    // HTMX events
    document.body.addEventListener('htmx:afterSwap', function(event) {
        // This fires after any HTMX content swap
        if (event.detail.target.id === 'auth-section') {
            // If login was successful and auth section was replaced
            document.getElementById('login-modal').style.display = 'none';
        }
    });

    document.body.addEventListener('htmx:beforeSwap', function(event) {
        console.log("Before swap:", event.detail);
        console.log("Target:", event.detail.target);
    });
    
    document.body.addEventListener('htmx:afterSwap', function(event) {
        console.log("After swap:", event.detail);
        console.log("Target:", event.detail.target);
        console.log("Response:", event.detail.xhr.responseText);
    });
});