/**
 * This is an adaptation of the original VRButton.
 * Original at:
 *      https://github.com/mrdoob/three.js/blob/dev/examples/jsm/webxr/VRButton.js
 */

/**
 * VRButton class that handles the creation of a VR session
 *
 * @class VRButton
 */
class VRButton {
  /**
   * Constructs a VRButton
   *
   * @static
   * @param {View} view - The aladin view
   * @return {HTMLButtonElement|HTMLAnchorElement} The VR mode button or an
   * error message
   */
  static createButton(view) {
    const button = document.createElement('button');

    /**
     * Function for handling the process of entering VR mode.
     */
    function showEnterVR(/* device*/) {
      let currentSession = null;

      /**
       * Callback function to handle when the XR session is started
       *
       * @param {XRSession} session - The XR session that has been started
       */
      async function onSessionStarted(session) {
        session.addEventListener('end', onSessionEnded);

        await view.options.vr.renderer.xr.setSession(session);
        button.textContent = 'EXIT VR';

        view.redrawVR();

        currentSession = session;
      }

      /**
       * Function to render the whole scene
       */
      function render() {
        // TODO Aladin rendering

        // External animation
        animation();
      }

      /**
       * Callback function to handle when the XR session ends
       */
      function onSessionEnded(/* event*/) {
        currentSession.removeEventListener('end', onSessionEnded);

        button.textContent = 'ENTER VR';

        currentSession = null;
      }

      //

      button.style.display = '';

      button.style.cursor = 'pointer';
      button.style.left = 'calc(50% - 50px)';
      button.style.width = '100px';

      button.textContent = 'ENTER VR';

      button.onmouseenter = function() {
        button.style.opacity = '1.0';
      };

      button.onmouseleave = function() {
        button.style.opacity = '0.5';
      };

      button.onclick = function() {
        if (currentSession === null) {
          // WebXR's requestReferenceSpace only works if the corresponding
          // feature was requested at session creation time. For simplicity,
          // just ask for the interesting ones as optional features, but be
          // aware that the requestReferenceSpace call will fail if it turns
          // out to be unavailable.
          // ('local' is always available for immersive sessions and doesn't
          // need to be requested separately.)

          const sessionInit = {optionalFeatures: ['local-floor', 'layers']};
          navigator.xr.requestSession(
              'immersive-vr', sessionInit).then(onSessionStarted);
        } else {
          currentSession.end();
        }
      };
    }

    /**
     * Function for disabling the VR mode button
     *
     * @param {HTMLButtonElement} button - The VR mode button element to
     * be disabled
     */
    function disableButton() {
      button.style.display = '';

      button.style.cursor = 'auto';
      button.style.left = 'calc(50% - 75px)';
      button.style.width = '150px';

      button.onmouseenter = null;
      button.onmouseleave = null;

      button.onclick = null;
    }

    /**
     * Function for handling the case where WebXR is not supported
     *
     * @description This function disables the VR mode button and displays a
     * message indicating that VR is not supported
     *
     * @param {HTMLButtonElement} button - The VR mode button element to be
     * disabled and updated with a message
     */
    function showWebXRNotFound() {
      disableButton();

      button.textContent = 'VR NOT SUPPORTED';
    }

    /**
     * Function for handling the case where VR is not allowed due to an
     * exception
     *
     * @description This function disables the VR mode button, logs an
     * exception to the console, and displays a message indicating that VR
     * is not allowed
     *
     * @param {any} exception - The exception object or error that indicates
     * why VR is not allowed
     * @param {HTMLButtonElement} button - The VR mode button element to be
     * disabled and updated with a message
     */
    function showVRNotAllowed(exception) {
      disableButton();

      console.warn('Exception when trying to call xr.isSessionSupported',
          exception);

      button.textContent = 'VR NOT ALLOWED';
    }

    /**
     * Function for styling an HTML element with specific CSS properties
     *
     * @param {HTMLElement} element - The HTML element to be styled
     */
    function stylizeElement(element) {
      element.style.position = 'absolute';
      element.style.bottom = '20px';
      element.style.padding = '12px 6px';
      element.style.border = '1px solid #fff';
      element.style.borderRadius = '4px';
      element.style.background = 'rgba(0,0,0,0.1)';
      element.style.color = '#fff';
      element.style.font = 'normal 13px sans-serif';
      element.style.textAlign = 'center';
      element.style.opacity = '0.5';
      element.style.outline = 'none';
      element.style.zIndex = '999';
    }

    if ('xr' in navigator) {
      button.id = 'VRButton';
      button.style.display = 'none';

      stylizeElement(button);

      navigator.xr.isSessionSupported('immersive-vr').then(function(supported) {
        supported ? showEnterVR() : showWebXRNotFound();

        if (supported && VRButton.xrSessionIsGranted) {
          button.click();
        }
      }).catch(showVRNotAllowed);

      return button;
    } else {
      const message = document.createElement('a');

      if (window.isSecureContext === false) {
        message.href = document.location.href.replace(/^http:/, 'https:');
        message.innerHTML = 'WEBXR NEEDS HTTPS';
      } else {
        message.href = 'https://immersiveweb.dev/';
        message.innerHTML = 'WEBXR NOT AVAILABLE';
      }

      message.style.left = 'calc(50% - 90px)';
      message.style.width = '180px';
      message.style.textDecoration = 'none';

      stylizeElement(message);

      return message;
    }
  }

  /**
   * Registers a listener for the "sessiongranted" event to track the XR
   * session being granted.
   *
   * @description This method checks if the WebXR API is available and
   * registers a listener for the "sessiongranted" event to track when an
   * XR session is granted. It sets the `VRButton.xrSessionIsGranted`
   * property to `true` when the event is triggered.
   */
  static registerSessionGrantedListener() {
    if ('xr' in navigator) {
      // WebXRViewer (based on Firefox) has a bug where addEventListener
      // throws a silent exception and aborts execution entirely.
      if (/WebXRViewer\//i.test(navigator.userAgent)) return;

      navigator.xr.addEventListener('sessiongranted', () => {
        VRButton.xrSessionIsGranted = true;
      });
    }
  }
}

VRButton.xrSessionIsGranted = false;
VRButton.registerSessionGrantedListener();

export {VRButton};
