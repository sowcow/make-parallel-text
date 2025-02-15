#
# commit rust code first
#
task :make do
  sh 'git archive --format=tar.gz HEAD:rust -o rust.tar.gz'
  sh 'docker build -t make-parallel-text:latest -f Dockerfile .'
  sh 'rm rust.tar.gz'
end

task :push do
  sh 'docker tag make-parallel-text:latest ghcr.io/sowcow/make-parallel-text:latest'
  sh 'docker push ghcr.io/sowcow/make-parallel-text:latest'
end
