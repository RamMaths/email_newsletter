sudo yum update -y
sudo amazon-linux-extras install docker -y
sudo service docker start
sudo usermod -a -G docker ec2-user
sudo yum install git -y
mkdir downloads
cd downloads
git clone https://github.com/RamMaths/email_newsletter.git
cd email_newsletter
