# HTTP File Runner Documentation

This directory contains the complete documentation website for HTTP File Runner, built as a modern static website with HTML, CSS, and JavaScript.

## 🌐 Live Documentation

The documentation is automatically deployed to GitHub Pages: 
**[https://christianhelle.github.io/httprunner/](https://christianhelle.github.io/httprunner/)**

## 📁 Structure

```
docs/
├── index.html              # Homepage with overview and quick start
├── guide.html              # Complete user guide
├── reference.html          # API reference documentation
├── CI-CD-SETUP.html        # CI/CD pipeline setup guide
├── DOCKER-SETUP.html       # Docker container setup guide
├── styles.css              # Main stylesheet with modern design
├── script.js               # JavaScript for interactivity
├── CI-CD-SETUP.md          # Original Markdown (kept for reference)
├── DOCKER-SETUP.md         # Original Markdown (kept for reference)
├── SNAPCRAFT.md            # Snapcraft setup documentation
└── README.md               # This file
```

## 🎨 Features

### Modern Design
- **Responsive Layout**: Works perfectly on desktop, tablet, and mobile
- **Dark/Light Theme Support**: Automatic theme detection with manual override
- **Interactive Navigation**: Smooth scrolling, active section highlighting
- **Professional Typography**: Clean, readable fonts with proper hierarchy

### User Experience
- **Copy-to-Clipboard**: Click any code block to copy commands
- **Table of Contents**: Sticky navigation sidebar for easy browsing
- **Search-Friendly**: Semantic HTML with proper meta tags
- **Fast Loading**: Optimized CSS and minimal JavaScript

### Content Organization
- **Homepage**: Quick overview with feature highlights and installation options
- **User Guide**: Step-by-step instructions for all features
- **API Reference**: Complete command-line and file format documentation
- **Setup Guides**: Detailed CI/CD and Docker deployment instructions

## 🚀 Development

### Local Development
To work on the documentation locally:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/christianhelle/httprunner.git
   cd httprunner/docs
   ```

2. **Serve locally**:
   ```bash
   # Using Python
   python -m http.server 8000
   
   # Using Node.js
   npx serve .
   
   # Using PHP
   php -S localhost:8000
   ```

3. **Open in browser**: Visit `http://localhost:8000`

### File Editing
- **HTML Files**: Update content, structure, and links
- **CSS (styles.css)**: Modify styling, colors, layout, and responsive design
- **JavaScript (script.js)**: Add interactivity and dynamic features

### Content Updates
When updating documentation:

1. **Edit HTML files** for content changes
2. **Update navigation** in all files if adding new pages
3. **Test responsiveness** on different screen sizes
4. **Validate HTML** and check for broken links
5. **Commit changes** - GitHub Pages will auto-deploy

## 📋 Maintenance

### Regular Updates
- **Version Information**: Update version numbers in examples
- **Feature Documentation**: Add new features as they're implemented
- **Link Validation**: Check external links periodically
- **Browser Testing**: Test on different browsers and devices

### Adding New Pages
1. Create new HTML file in `docs/` directory
2. Follow existing structure and styling
3. Add navigation links to all existing pages
4. Update table of contents where applicable
5. Test all internal links

### Style Customization
The `styles.css` file uses CSS custom properties (variables) for easy theming:

```css
:root {
    --primary-color: #2563eb;
    --success-color: #10b981;
    --error-color: #ef4444;
    /* ... more variables */
}
```

## 🔧 Technical Details

### CSS Framework
- **Custom CSS**: No external frameworks for faster loading
- **CSS Grid & Flexbox**: Modern layout techniques
- **CSS Variables**: Easy theme customization
- **Media Queries**: Responsive breakpoints for all devices

### JavaScript Features
- **Vanilla JavaScript**: No dependencies for better performance
- **Progressive Enhancement**: Works without JavaScript
- **Smooth Scrolling**: Enhanced navigation experience
- **Copy Functionality**: Easy code copying

### Accessibility
- **Semantic HTML**: Proper heading hierarchy and landmarks
- **Keyboard Navigation**: Full keyboard support
- **Screen Reader Friendly**: ARIA labels and descriptions
- **Color Contrast**: WCAG AA compliant colors

### Performance
- **Optimized Images**: Proper compression and formats
- **Minimal Dependencies**: Self-contained CSS and JavaScript
- **Critical CSS**: Above-the-fold styling prioritized
- **Lazy Loading**: Images load as needed

## 🚀 Deployment

### Automatic Deployment
The documentation is automatically deployed via GitHub Actions:

- **Trigger**: Push to `main` branch with changes to `docs/` folder
- **Workflow**: `.github/workflows/docs.yml`
- **Target**: GitHub Pages at `https://christianhelle.github.io/httprunner/`

### Manual Deployment
If needed, you can manually trigger deployment:

1. Go to GitHub repository → Actions
2. Select "Deploy Documentation to GitHub Pages"
3. Click "Run workflow"

### Custom Domain (Optional)
To use a custom domain:

1. Add `CNAME` file to `docs/` directory with your domain
2. Configure DNS settings in your domain provider
3. Enable custom domain in repository Settings → Pages

## 📖 Content Guidelines

### Writing Style
- **Clear and Concise**: Use simple, direct language
- **Example-Driven**: Include practical examples for all features
- **Step-by-Step**: Break complex tasks into numbered steps
- **Consistent Terminology**: Use the same terms throughout

### Code Examples
- **Complete Examples**: Show full command lines and file contents
- **Platform-Specific**: Include Windows, macOS, and Linux variations
- **Copy-Friendly**: Format for easy copying and pasting
- **Tested**: Verify all examples work as documented

### Visual Design
- **Consistent Layout**: Follow established patterns
- **Proper Hierarchy**: Use headings appropriately (h1 → h2 → h3)
- **Visual Cues**: Use colors and icons meaningfully
- **Whitespace**: Ensure good readability with proper spacing

## 🤝 Contributing

### Content Contributions
1. **Fork the repository**
2. **Make changes** to relevant HTML files in `docs/`
3. **Test locally** to ensure everything works
4. **Submit pull request** with clear description

### Reporting Issues
- **Documentation Issues**: Use GitHub Issues with "documentation" label
- **Broken Links**: Report with specific page and link
- **Content Suggestions**: Propose improvements or missing topics

### Code of Conduct
Please follow the project's code of conduct when contributing to documentation.

## 📄 License

This documentation is part of the HTTP File Runner project and is licensed under the MIT License. See the main repository's LICENSE file for details.

---

**Built with ❤️ for the HTTP File Runner community**
