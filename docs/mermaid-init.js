// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

(() => {
    const darkThemes = ['ayu', 'navy', 'coal'];
    const lightThemes = ['light', 'rust'];

    const classList = document.getElementsByTagName('html')[0].classList;

    let lastThemeWasLight = true;
    for (const cssClass of classList) {
        if (darkThemes.includes(cssClass)) {
            lastThemeWasLight = false;
            break;
        }
    }

    const theme = lastThemeWasLight ? 'default' : 'dark';
    mermaid.initialize({
        startOnLoad: true,
        theme,
        themeVariables: {
            primaryColor: '#BB86FC',
            primaryTextColor: '#000',
            primaryBorderColor: '#7C4DFF',
            lineColor: '#F5F5F5',
            secondaryColor: '#03DAC6',
            tertiaryColor: '#3700B3',
            background: '#FFFFFF',
            mainBkg: '#E8EAF6',
            secondBkg: '#F3E5F5',
            mainContrastColor: 'darkgrey',
            darkMode: !lastThemeWasLight,
            fontSize: '16px',
            fontFamily: '"Inter", "Segoe UI", "Roboto", "Helvetica", "Arial", sans-serif'
        }
    });

    // Simplest way to make mermaid re-render the diagrams in the new theme is via refreshing the page

    for (const darkTheme of darkThemes) {
        document.getElementById(darkTheme).addEventListener('click', () => {
            if (lastThemeWasLight) {
                window.location.reload();
            }
        });
    }

    for (const lightTheme of lightThemes) {
        document.getElementById(lightTheme).addEventListener('click', () => {
            if (!lastThemeWasLight) {
                window.location.reload();
            }
        });
    }
})();
