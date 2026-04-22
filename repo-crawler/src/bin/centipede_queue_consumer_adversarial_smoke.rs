use repo_crawler::centipede_queue_consumer_adversarial::run_adversarial_checks;

fn main() {
    if let Err(err) = run_adversarial_checks() {
        eprintln!("centipede_queue_consumer_adversarial_smoke: {err}");
        std::process::exit(1);
    }

    println!("centipede_queue_consumer_adversarial_smoke: ok");
}
