import {Utils} from '@/js/Utils.ts';

describe('Utils.ts', () => {
    beforeEach(() => {
        //delete window.location;
        window.location = {href: {}, search: ''};
    });

    it('correctly parse a location parameter', () => {
        window.location.search = '?survey=DSS';
        expect(Utils.urlParam('survey')).toEqual('DSS');
    });
});
