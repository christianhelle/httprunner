# Contributing to HTTP File Runner Documentation

Thank you for your interest in improving the HTTP File Runner documentation! This guide will help you contribute effectively.

🌐 **[Try the WASM app online](https://christianhelle.com/httprunner/app/)** - No installation required!

## 📝 Documentation Structure

Our documentation website is built with:
- **HTML5** for structure and content
- **CSS3** (vanilla, no frameworks) for styling
- **Vanilla JavaScript** for interactivity
- **GitHub Pages** for hosting

## 🚀 Quick Start

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/your-username/httprunner.git
   cd httprunner/docs
   ```

2. **Local Development**:
   ```bash
   # Using Python
   python -m http.server 8000
   
   # Or using Node.js
   npx serve .
   ```

3. **Open Browser**: Visit `http://localhost:8000`

## 📁 File Organization

```
docs/
├── index.html              # Homepage
├── guide.html              # User guide
├── reference.html          # API reference
├── CI-CD-SETUP.html        # CI/CD documentation
├── DOCKER-SETUP.html       # Docker setup guide
├── styles.css              # Main stylesheet
├── script.js               # JavaScript functionality
├── 404.html                # Custom 404 page
├── sitemap.xml             # SEO sitemap
├── robots.txt              # Search engine directives
└── _config.yml             # GitHub Pages config
```

## ✏️ Making Changes

### Content Updates

1. **Edit HTML Files**: Update content directly in the appropriate HTML file
2. **Follow Structure**: Maintain consistent HTML structure and classes
3. **Update Navigation**: If adding new pages, update nav in all files
4. **Test Locally**: Always test changes in a local server

### Style Changes

1. **Edit styles.css**: All styling is in one file
2. **Use CSS Variables**: Modify the `:root` variables for theme changes
3. **Maintain Responsiveness**: Test on different screen sizes
4. **Follow Naming**: Use existing class naming conventions

### Adding New Pages

1. **Create HTML File**: Follow the structure of existing pages
2. **Add Navigation**: Update the nav menu in all existing pages
3. **Update Sitemap**: Add new pages to `sitemap.xml`
4. **Test All Links**: Ensure all internal links work

## 🎨 Design Guidelines

### Visual Consistency
- Use the established color scheme (defined in CSS variables)
- Maintain consistent spacing and typography
- Follow the existing layout patterns
- Ensure proper contrast for accessibility

### Content Guidelines
- **Clear Headlines**: Use descriptive, hierarchical headings
- **Code Examples**: Include practical, tested examples
- **Step-by-Step**: Break complex processes into numbered steps
- **Consistent Terminology**: Use the same terms throughout

### Responsive Design
- Test on mobile, tablet, and desktop
- Ensure readable text sizes on all devices
- Maintain usable navigation on small screens
- Test touch interactions on mobile

## 🧪 Testing Checklist

Before submitting changes:

- [ ] **Local Testing**: Verify all changes work locally
- [ ] **Link Validation**: Check all internal and external links
- [ ] **Mobile Testing**: Test on mobile devices/browser dev tools
- [ ] **Cross-Browser**: Test on Chrome, Firefox, Safari, Edge
- [ ] **Accessibility**: Check keyboard navigation and screen readers
- [ ] **Loading Speed**: Ensure pages load quickly
- [ ] **HTML Validation**: Validate HTML structure
- [ ] **Spell Check**: Proofread all new content

## 📋 Content Types

### User Guide Content
- Step-by-step tutorials
- Feature explanations with examples
- Best practices and tips
- Troubleshooting guides

### Reference Content
- Command-line options
- File format specifications
- API documentation
- Configuration options

### Setup Guides
- Installation instructions
- Environment configuration
- Integration examples
- Deployment guides

## 🔧 Technical Specifications

### HTML Requirements
- Use semantic HTML5 elements
- Include proper meta tags
- Maintain heading hierarchy (h1 → h2 → h3)
- Add alt text for images

### CSS Guidelines
- Use CSS custom properties for theming
- Mobile-first responsive design
- Avoid inline styles
- Comment complex styling

### JavaScript Standards
- Use vanilla JavaScript (no dependencies)
- Progressive enhancement (works without JS)
- Add comments for complex functions
- Handle errors gracefully

## 🚀 Deployment

### Automatic Deployment
- **Trigger**: Push to `main` branch with changes to `docs/`
- **Workflow**: `.github/workflows/docs.yml`
- **URL**: `https://christianhelle.github.io/httprunner/`

### Manual Testing
Test deployment locally before pushing:

```bash
# Simulate the deployed environment
cd docs
python -m http.server 8000 --bind 127.0.0.1
```

## 📊 Analytics and SEO

### SEO Optimization
- Include relevant meta descriptions
- Use semantic HTML structure
- Add structured data where appropriate
- Optimize images with proper alt text

### Performance
- Minimize CSS and JavaScript
- Optimize images
- Use efficient loading strategies
- Monitor Core Web Vitals

## 🤝 Contribution Process

### Small Changes
1. Fork the repository
2. Make changes directly to the HTML/CSS files
3. Test locally
4. Submit a pull request

### Large Changes
1. **Open an Issue**: Discuss major changes first
2. **Create Branch**: Use descriptive branch names
3. **Make Changes**: Follow the guidelines above
4. **Test Thoroughly**: Use the full testing checklist
5. **Submit PR**: Include screenshots and detailed description

### Pull Request Guidelines
- **Descriptive Title**: Clearly describe the changes
- **Detailed Description**: Explain what and why
- **Screenshots**: Include before/after images for visual changes
- **Testing Notes**: Describe testing performed
- **Breaking Changes**: Highlight any breaking changes

## 🐛 Reporting Issues

### Documentation Issues
- **Content Errors**: Factual mistakes or outdated information
- **Broken Links**: Internal or external links that don't work
- **Display Issues**: Layout problems or visual bugs
- **Accessibility Issues**: Problems with screen readers or keyboard navigation

### Issue Template
```markdown
**Page/Section**: [URL or page name]
**Issue Type**: [Content/Link/Display/Accessibility]
**Description**: [Clear description of the issue]
**Browser/Device**: [If relevant]
**Steps to Reproduce**: [If applicable]
**Expected Behavior**: [What should happen]
**Screenshots**: [If helpful]
```

## 🏆 Recognition

Contributors will be recognized in:
- GitHub contributor stats
- Project acknowledgments
- Community discussions

Thank you for helping make HTTP File Runner documentation better! 🚀
