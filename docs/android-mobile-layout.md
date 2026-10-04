# Android Mobile Layout & Target Compliance

## Mobile Layout Rules
1. **TitleBar Hidden**: On Android/iOS, the desktop frameless TitleBar is hidden automatically via responsive detection (`window.__TAURI_INTERNALS__` and screen width queries).
2. **Navigation Drawer**: Mobile uses bottom sheet or sidebar drawer navigation for easy thumb reach.
3. **Touch Targets**: All interactive elements (buttons, inputs, toggle switches, PIN buttons) maintain a minimum touch target bounding box of **44x44 px** according to WCAG 2.5.5 and Android Material Guidelines.
4. **PIN Pad**: Custom responsive PIN pad component designed for mobile touch screens.
