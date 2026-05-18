export class Widget {
    constructor(name) {
        this.name = name;
    }

    render(target) {
        return `${target}:${this.name}`;
    }
}

export function makeWidget(name) {
    return new Widget(name);
}
