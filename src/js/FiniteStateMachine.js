export class FSM {
    // Constructor
    constructor(options) {
        this.state = options && options.state;
        this.transitions = options && options.transitions || {};
    }

    // Do nothing if the to is inaccesible
    dispatch(to, params) {
        const action = this.transitions[this.state][to];
        if (action) {
            this.state = to;

            if (params) {
                action(params);
            } else {
                action()
            }
        }
    }
}