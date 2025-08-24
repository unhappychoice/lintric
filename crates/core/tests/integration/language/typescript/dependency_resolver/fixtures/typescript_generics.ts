interface Repository<T> {
    save(item: T): void;
    findById(id: string): T | null;
}

class UserRepository implements Repository<User> {
    private users: User[] = [];
    
    save(user: User): void {
        this.users.push(user);
    }
    
    findById(id: string): User | null {
        return this.users.find(u => u.id === id) || null;
    }
}

interface User {
    id: string;
    name: string;
}

function processItems<T>(items: T[]): T[] {
    return items.filter(item => item !== null);
}

function main() {
    const repo = new UserRepository();
    repo.save({ id: "1", name: "John" });
    
    const user = repo.findById("1");
    const users = [user];
    const filtered = processItems(users);
}