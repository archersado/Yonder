const spaces = await listTaskSpaces();
console.log(JSON.stringify(spaces.map(({ id, name, ownership, status }) => ({ id, name, ownership, status }))));
