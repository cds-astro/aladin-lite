import { A } from '../../src/js/A.js';
import { expect } from 'vitest';
import { Window } from 'happy-dom';

it("Initializes Aladin with default parameters", () => {
    const window = new Window({ url: 'https://localhost:8080' });
    const document = window.document;
    document.body.innerHTML = '<div id="aladin-lite-div" style="width: 500px; height: 500px"></div>';
    A.init.then( () => {
        A.aladin("#aladin-lite-div");
    }
    )
    expect(window).toMatchSnapshot();
    window.close();
})