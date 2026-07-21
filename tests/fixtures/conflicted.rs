pub fn before() -> usize {
    1
}
/* Documentation example under active merge:
<<<<<<< Conflict 1 of 1
%%%%%%% Changes from base to side #1
-let greeting = "hello";
+let greeting = "zdravo";
+++++++ Contents of side #2
let greeting = "bonjour";
>>>>>>> Conflict 1 of 1 ends
*/
pub fn after() -> usize {
    5
}
