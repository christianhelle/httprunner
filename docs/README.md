# HTTP File Runner Documentation

This directory contains supplementary documentation and the static website for HTTP File Runner.

The main [README.md](../README.md) in the repository root is the primary and most detailed documentation source.

## Live Documentation

- **Website**: [https://christianhelle.github.io/httprunner/](https://christianhelle.github.io/httprunner/)
- **WASM App**: [https://christianhelle.com/httprunner/app/](https://christianhelle.com/httprunner/app/)

## Structure

```
docs/
├── website/                    # Static website (deployed to GitHub Pages)
│   ├── index.html              # Homepage
│   ├── guide.html              # User guide
│   ├── reference.html          # API reference
│   ├── styles.css              # Stylesheet
│   ├── script.js               # JavaScript
│   └── ...                     # Other HTML pages and assets
├── DOCKER-SETUP.md             # Docker container publishing and registry setup
├── DOCKER-TROUBLESHOOTING.md   # Docker networking troubleshooting guide
├── SNAPCRAFT.md                # Snapcraft packaging and distribution guide
└── README.md                   # This file
```

## Deployment

The website is automatically deployed via GitHub Actions (`.github/workflows/docs.yml`) on push to `main` when files in `docs/website/` change.
