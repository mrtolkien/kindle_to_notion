pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    const single_clip: &str = "How to Win Friends and Influence People (Dale Carnegie)
- Your Highlight at location 1502-1507 | Added on Tuesday, 1 December 2020 16:58:58

The old neighbour called at the White House, and Lincoln talked to him for hours about the advisability of issuing a proclamation freeing the slaves. Lincoln went over all the arguments for and against such a move, and then read letters and newspaper articles, some denouncing him for not freeing the slaves and others denouncing him for fear he was going to free them. After talking for hours, Lincoln shook hands with his old neighbour, said good night, and sent him back to Illinois without even asking for his opinion. Lincoln had done all the talking himself. That seemed to clarify his mind. ‘He seemed to feel easier after that talk,’ the old friend said. Lincoln hadn’t wanted advice. He had wanted merely a friendly, sympathetic listener to whom he could unburden himself.
";

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
