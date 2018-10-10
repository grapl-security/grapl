import subprocess

project_name="engagement-creation-service"
local_deps = [
    "../incident-graph/",
    "../graph-descriptions/",
    "../sqs-microservice/",
]


def main():
    for dep in local_deps:
        print("Copying in {}".format(dep))
        subprocess.check_call(  ["cp", "-r", dep, "."])
    print("running docker build")
    print(subprocess.check_call([r'docker run --rm -it -v "$(pwd)":/home/rust/src -t 3b07546503c6 cargo build --release'],
                                  shell=True,
                                  stderr=subprocess.STDOUT
                                  ))
    print("cp")
    subprocess.check_call(["cp", "./target/x86_64-unknown-linux-musl/release/{}".format(project_name), "."])
    print("zip")
    subprocess.check_call(["zip", "./{}.zip".format(project_name), "./{}".format(project_name)])
    print("cp")
    subprocess.check_call(["cp", "./{}.zip".format(project_name), " ~/workspace/grapl/grapl-cdk/"])
    print("rm")
    subprocess.check_call(["rm", "./{}.zip".format(project_name)])
    # pass


if __name__ == '__main__':
    main()