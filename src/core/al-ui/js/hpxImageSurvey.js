
import { HiPSDefinition } from "../../../../../js/HiPSDefinition.js";
import { HpxImageSurvey } from "../../../../../js/HpxImageSurvey.js";
import { Utils } from "../../../../../js/Utils.js";

export async function fetchSurveyMetadata(rootURL) {
    // Use the url for retrieving the HiPS properties
    // remove final slash
    if (rootURL.slice(-1) === '/') {
        rootURL = rootURL.substr(0, rootURL.length-1);
    }

    // make URL absolute
    rootURL = Utils.getAbsoluteURL(rootURL);

    // fast fix for HTTPS support --> will work for all HiPS served by CDS
    if (Utils.isHttpsContext() && ( /u-strasbg.fr/i.test(rootURL) || /unistra.fr/i.test(rootURL)  ) ) {
        rootURL = rootURL.replace('http://', 'https://');
    }

    const url = rootURL + '/properties';
    let metadata = await fetch(url)
        .then((response) => response.text());
    // We get the property here
    metadata = HiPSDefinition.parseHiPSProperties(metadata);

    // 1. Ensure there is exactly one survey matching
    if (!metadata) {
        throw 'no surveys matching';
    }
    //console.log(metadata);
    return metadata;
}
