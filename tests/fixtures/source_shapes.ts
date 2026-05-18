export interface Repository<T> {
    get(id: string): Promise<T>;
}

export type User = {
    id: string;
    name: string;
};

export class UserService {
    constructor(private readonly repo: Repository<User>) {}

    async load(id: string): Promise<User> {
        return this.repo.get(id);
    }
}

export function formatUser(user: User): string {
    return `${user.id}:${user.name}`;
}
