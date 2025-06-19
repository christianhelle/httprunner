// Mobile navigation toggle
document.addEventListener('DOMContentLoaded', function() {
    const hamburger = document.querySelector('.hamburger');
    const navMenu = document.querySelector('.nav-menu');
    
    if (hamburger && navMenu) {
        hamburger.addEventListener('click', function() {
            navMenu.classList.toggle('active');
            hamburger.classList.toggle('active');
        });
    }
    
    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
    
    // Copy code blocks on click
    document.querySelectorAll('.code-block, .install-code').forEach(block => {
        block.addEventListener('click', async function() {
            const code = this.querySelector('code');
            if (code) {
                try {
                    await navigator.clipboard.writeText(code.textContent);
                    
                    // Show feedback
                    const originalTitle = this.title;
                    this.title = 'Copied!';
                    this.style.background = '#10b981';
                    this.style.transition = 'background 0.2s';
                    
                    setTimeout(() => {
                        this.title = originalTitle || 'Click to copy';
                        this.style.background = '';
                    }, 1000);
                } catch (err) {
                    console.error('Failed to copy text: ', err);
                }
            }
        });
        
        // Add cursor pointer and title
        block.style.cursor = 'pointer';
        block.title = 'Click to copy';
    });
    
    // Highlight current section in navigation
    const sections = document.querySelectorAll('section[id]');
    const navLinks = document.querySelectorAll('.nav-link[href^="#"]');
    
    function highlightNavigation() {
        let currentSection = '';
        sections.forEach(section => {
            const rect = section.getBoundingClientRect();
            if (rect.top <= 100 && rect.bottom >= 100) {
                currentSection = section.id;
            }
        });
        
        navLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('href') === `#${currentSection}`) {
                link.classList.add('active');
            }
        });
    }
    
    window.addEventListener('scroll', highlightNavigation);
    highlightNavigation(); // Initial call
    
    // Add animation on scroll
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);
    
    // Observe feature cards and install cards
    document.querySelectorAll('.feature-card, .install-card, .usage-example').forEach(card => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(20px)';
        card.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(card);
    });
    
    // TOC functionality for guide pages
    initializeTOC();
});

function initializeTOC() {
    const tocToggle = document.getElementById('tocToggle');
    const guideSidebar = document.getElementById('guideSidebar');
    const tocLinks = document.querySelectorAll('.guide-sidebar .toc a');
    const sections = document.querySelectorAll('.guide-section');
    
    if (!tocToggle || !guideSidebar) return;
    
    // Create overlay for mobile
    const overlay = document.createElement('div');
    overlay.className = 'sidebar-overlay';
    overlay.id = 'sidebarOverlay';
    document.body.appendChild(overlay);
    
    // Toggle sidebar
    tocToggle.addEventListener('click', function() {
        guideSidebar.classList.toggle('active');
        overlay.classList.toggle('active');
    });
    
    // Close sidebar when clicking overlay
    overlay.addEventListener('click', function() {
        guideSidebar.classList.remove('active');
        overlay.classList.remove('active');
    });
    
    // Close sidebar on mobile when clicking a link
    tocLinks.forEach(link => {
        link.addEventListener('click', function() {
            if (window.innerWidth <= 768) {
                guideSidebar.classList.remove('active');
                overlay.classList.remove('active');
            }
        });
    });
    
    // Highlight active section in TOC
    function updateActiveTOCLink() {
        let activeSection = '';
        
        sections.forEach(section => {
            const rect = section.getBoundingClientRect();
            const sectionTop = rect.top + window.scrollY;
            const sectionBottom = sectionTop + rect.height;
            const scrollPosition = window.scrollY + 150; // Offset for header
            
            if (scrollPosition >= sectionTop && scrollPosition < sectionBottom) {
                activeSection = section.id;
            }
        });
        
        // If we're at the top of the page, highlight the first section
        if (window.scrollY < 200 && sections.length > 0) {
            activeSection = sections[0].id;
        }
        
        // Update TOC links
        tocLinks.forEach(link => {
            link.classList.remove('active');
            const href = link.getAttribute('href');
            if (href === `#${activeSection}`) {
                link.classList.add('active');
            }
        });
    }
    
    // Listen for scroll events
    let scrollTimer;
    window.addEventListener('scroll', function() {
        if (scrollTimer) {
            clearTimeout(scrollTimer);
        }
        scrollTimer = setTimeout(updateActiveTOCLink, 10);
    });
    
    // Initial call
    updateActiveTOCLink();
    
    // Handle window resize
    window.addEventListener('resize', function() {
        if (window.innerWidth > 768) {
            overlay.classList.remove('active');
        }
    });
}

// Add styles for mobile navigation
const style = document.createElement('style');
style.textContent = `
    .nav-menu.active {
        display: flex !important;
        position: fixed;
        top: 100%;
        left: 0;
        width: 100%;
        background: var(--bg-color);
        flex-direction: column;
        padding: 1rem 2rem;
        border-top: 1px solid var(--border-color);
        box-shadow: var(--shadow-lg);
        z-index: 99;
    }
    
    .hamburger.active span:nth-child(1) {
        transform: rotate(-45deg) translate(-5px, 6px);
    }
    
    .hamburger.active span:nth-child(2) {
        opacity: 0;
    }
    
    .hamburger.active span:nth-child(3) {
        transform: rotate(45deg) translate(-5px, -6px);
    }
    
    .nav-link.active {
        color: var(--primary-color) !important;
        position: relative;
    }
    
    .nav-link.active::after {
        content: '';
        position: absolute;
        bottom: -5px;
        left: 0;
        width: 100%;
        height: 2px;
        background: var(--primary-color);
        border-radius: 1px;
    }
    
    @media (max-width: 768px) {
        .nav-link.active::after {
            display: none;
        }
    }
`;
document.head.appendChild(style);
